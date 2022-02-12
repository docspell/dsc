//! A http client for Docspell.
//!
//! This provides a http client to Docspell based on reqwest. It
//! implements the endpoints as described
//! [here](https://docspell.org/openapi/docspell-openapi.html).
//!
//! # Usage
//!
//! ```rust
//! let client = dsc::http::Client::new("http://localhost:7880").unwrap();
//! println!("{:?}", client.version());
//! ```
//!
//! For multiple requests, it is recommended to reuse one client to
//! benefit from connection pooling used in the underlying
//! [`reqwest::blocking::Client`].
//!
//! # Authentication
//!
//! The `login` method can be used to perform a login. The returned
//! session is stored in the user's home directory and used for all
//! secured requests where no explicit token is supplied.
//!
//! When dealing with files (upload or check if a file exists),
//! besides a valid session, a [source
//! id](https://docspell.org/docs/webapp/uploading/#anonymous-upload)
//! or the [integration
//! endpoint](https://docspell.org/docs/api/upload/#integration-endpoint)
//! can be used.
//!
//! # Admin
//!
//! There are some commands that require the [admin
//! secret](https://docspell.org/docs/configure/#admin-endpoint) from
//! Docspells configuration file.

pub mod payload;
mod session;
mod util;

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use self::payload::*;
use self::util::{DOCSPELL_ADMIN, DOCSPELL_AUTH};
use reqwest::blocking::{
    multipart::{Form, Part},
    ClientBuilder, RequestBuilder, Response,
};
use reqwest::StatusCode;
use snafu::{ResultExt, Snafu};

const APP_JSON: &str = "application/json";
const ID_LEN: usize = 47;
const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// The errors cases.
#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An error was received from {}: {}", url, source))]
    Http { source: reqwest::Error, url: String },

    #[snafu(display("An error parsing mime '{}': {}", raw, source))]
    Mime { source: reqwest::Error, raw: String },

    #[snafu(display("An error occurred creating the http client: {}", source))]
    ClientCreate { source: reqwest::Error },

    #[snafu(display("Session error: {}", source))]
    Session { source: self::session::Error },

    #[snafu(display("An error occured serializing the response: {}", source))]
    SerializeResp { source: reqwest::Error },

    #[snafu(display("An error occured serializing the request: {}", source))]
    SerializeReq { source: serde_json::Error },

    #[snafu(display("Login failed"))]
    LoginFailed,

    #[snafu(display("Authentication failure for integration endpoint: {}", url))]
    IntEndpointAuth { url: String },

    #[snafu(display("Unexpected response status: {}", status))]
    UnexpectedStatus { status: u16, url: String },

    #[snafu(display("Error opening file '{}': {}", path.display(), source))]
    OpenFile {
        source: std::io::Error,
        path: PathBuf,
    },

    #[snafu(display("No item found for: {}", id))]
    ItemNotFound { id: String },

    #[snafu(display("Item id not unique: {}", id))]
    ItemNotUnique { id: String },
}

/// The docspell http client.
///
/// This wraps a `reqwest::blocking::Client` with methods
/// corresponding to an api endpoint. The required parameter for
/// construction is the base docspell url, something like
/// `http://localhost:7880` that is used to base all urls on.
///
/// Note that this client handles the session token by storing it
/// beneath the current user's home directory.
pub struct Client {
    client: reqwest::blocking::Client,
    base_url: String,
}

impl Client {
    /// Create a new client by providing the base url to docspell. For
    /// example: `http://localhost:7880`.
    pub fn new<S: Into<String>>(docspell_url: S) -> Result<Client, Error> {
        let url = docspell_url.into();
        log::info!("Create docspell client for: {}", url);
        let client = ClientBuilder::new()
            .user_agent(APP_USER_AGENT)
            .build()
            .context(ClientCreateSnafu)?;
        Ok(Client {
            client,
            base_url: url,
        })
    }

    /// Queries the Docspell server for its version and build information.
    pub fn version(&self) -> Result<VersionInfo, Error> {
        let url = &format!("{}/api/info/version", self.base_url);
        self.client
            .get(url)
            .send()
            .context(HttpSnafu { url })?
            .json::<VersionInfo>()
            .context(SerializeRespSnafu)
    }

    /// Login to Docspell returning the session token that must be
    /// used with all secured requests.
    ///
    /// This token is stored in the filesystem and will be used as a
    /// fallback if no specific token is supplied.
    ///
    /// If the account is configured for two-factor authentication,
    /// the next call must be to send the confirmation code together
    /// with the token returned here.
    pub fn login(&self, req: &AuthRequest) -> Result<AuthResp, Error> {
        let url = &format!("{}/api/v1/open/auth/login", self.base_url);
        let result = self
            .client
            .post(url)
            .json(req)
            .send()
            .and_then(|r| r.error_for_status())
            .context(HttpSnafu { url })?
            .json::<AuthResp>()
            .context(SerializeRespSnafu)?;

        if result.success {
            session::store_session(&result).context(SessionSnafu)?;
            Ok(result)
        } else {
            log::debug!("Login result: {:?}", result);
            Err(Error::LoginFailed)
        }
    }

    /// Login to Docspell returning the session token that must be
    /// used with all secured requests. This is providing the second
    /// factor.
    ///
    /// This token is stored in the filesystem and will be used as a
    /// fallback if no specific token is supplied.
    pub fn login_otp(&self, otp: &str) -> Result<AuthResp, Error> {
        let url = &format!("{}/api/v1/open/auth/two-factor", self.base_url);
        let token = session::session_token_from_file().context(SessionSnafu)?;
        let req = SecondFactor {
            otp: otp.to_string(),
            token,
            remember_me: false,
        };
        log::debug!("Sending second factor: {:?}", req);
        let result = self
            .client
            .post(url)
            .json(&req)
            .send()
            .and_then(|r| r.error_for_status())
            .context(HttpSnafu { url })?
            .json::<AuthResp>()
            .context(SerializeRespSnafu)?;

        if result.success {
            session::store_session(&result).context(SessionSnafu)?;
            Ok(result)
        } else {
            log::debug!("Login result: {:?}", result);
            Err(Error::LoginFailed)
        }
    }

    /// Performs a logout by deleting the current session information.
    pub fn logout(&self) -> Result<(), Error> {
        session::drop_session().context(SessionSnafu)
    }

    /// Performs a login via a session token. It returns a new session
    /// token with a fresh lifetime.
    pub fn session_login(&self, token: &str) -> Result<AuthResp, Error> {
        let url = &format!("{}/api/v1/sec/auth/session", self.base_url);
        let result = self
            .client
            .post(url)
            .header(DOCSPELL_AUTH, token)
            .send()
            .and_then(|r| r.error_for_status())
            .context(HttpSnafu { url })?
            .json::<AuthResp>()
            .context(SerializeRespSnafu)?;

        if result.success {
            Ok(result)
        } else {
            log::debug!("Session login result: {:?}", result);
            Err(Error::LoginFailed)
        }
    }

    /// Searches for documents using the given query. See [the query
    /// documentation](https://docspell.org/docs/query/) for
    /// information about the query.
    ///
    /// If `token` is specified, it is used to authenticate. Otherwise
    /// a stored session is used.
    pub fn search(&self, token: &Option<String>, req: &SearchReq) -> Result<SearchResult, Error> {
        let url = &format!("{}/api/v1/sec/item/search", self.base_url);
        let token = session::session_token(token, self).context(SessionSnafu)?;
        self.client
            .get(url)
            .header(DOCSPELL_AUTH, token)
            .query(&[
                ("limit", &req.limit.to_string()),
                ("offset", &req.offset.to_string()),
                ("withDetails", &req.with_details.to_string()),
                ("q", &req.query),
                ("searchMode", &req.search_mode.as_str().to_string()),
            ])
            .send()
            .and_then(|r| r.error_for_status())
            .context(HttpSnafu { url })?
            .json::<SearchResult>()
            .context(SerializeRespSnafu)
    }

    /// Returns a summary for a given search query.
    ///
    /// If `token` is specified, it is used to authenticate. Otherwise
    /// a stored session is used.
    pub fn summary<S: Into<String>>(
        &self,
        token: &Option<String>,
        query: S,
    ) -> Result<Summary, Error> {
        let url = &format!("{}/api/v1/sec/item/searchStats", self.base_url);
        let token = session::session_token(token, self).context(SessionSnafu)?;
        self.client
            .get(url)
            .header(DOCSPELL_AUTH, token)
            .query(&[("q", &query.into())])
            .send()
            .and_then(|r| r.error_for_status())
            .context(HttpSnafu { url })?
            .json::<Summary>()
            .context(SerializeRespSnafu)
    }

    /// Lists all sources for the current user.
    ///
    /// If `token` is specified, it is used to authenticate. Otherwise
    /// a stored session is used.
    pub fn list_sources(&self, token: &Option<String>) -> Result<SourceList, Error> {
        let url = &format!("{}/api/v1/sec/source", self.base_url);
        let token = session::session_token(token, self).context(SessionSnafu)?;
        self.client
            .get(url)
            .header(DOCSPELL_AUTH, token)
            .send()
            .and_then(|r| r.error_for_status())
            .context(HttpSnafu { url })?
            .json::<SourceList>()
            .context(SerializeRespSnafu)
    }

    /// Lists all tags. The `query` argument may be a query for a
    /// name, which can contain the `*` wildcard at beginning or end.
    pub fn list_tags(&self, token: &Option<String>, query: &str) -> Result<TagList, Error> {
        let url = &format!("{}/api/v1/sec/tag", self.base_url);
        let token = session::session_token(token, self).context(SessionSnafu)?;
        self.client
            .get(url)
            .header(DOCSPELL_AUTH, token)
            .query(&[("q", query)])
            .send()
            .and_then(|r| r.error_for_status())
            .context(HttpSnafu { url })?
            .json::<TagList>()
            .context(SerializeRespSnafu)
    }

    /// Get all item details. The item is identified by its id. The id
    /// may be a prefix only, in this case another request is used to
    /// find the complete id.
    ///
    /// If `token` is specified, it is used to authenticate. Otherwise
    /// a stored session is used.
    pub fn get_item<S: AsRef<str>>(
        &self,
        token: &Option<String>,
        id: S,
    ) -> Result<Option<ItemDetail>, Error> {
        let item_id = self.complete_item_id(token, id.as_ref(), SearchMode::All)?;
        if let Some(iid) = item_id {
            let url = &format!("{}/api/v1/sec/item/{}", self.base_url, iid);
            let token = session::session_token(token, self).context(SessionSnafu)?;
            let resp = self
                .client
                .get(url)
                .header(DOCSPELL_AUTH, token)
                .send()
                .context(HttpSnafu { url })?;

            if resp.status() == StatusCode::NOT_FOUND {
                Ok(None)
            } else {
                resp.error_for_status()
                    .context(HttpSnafu { url })?
                    .json::<ItemDetail>()
                    .context(SerializeRespSnafu)
                    .map(Some)
            }
        } else {
            Ok(None)
        }
    }

    /// Adds the give tags to the item with the given id. The id may
    /// be given as a prefix, then another request is used to find the
    /// complete id.
    ///
    /// Tags can be given via their ids or names. They must exist or
    /// are ignored otherwise.
    pub fn link_tags<S: AsRef<str>>(
        &self,
        token: &Option<String>,
        id: S,
        tags: &StringList,
    ) -> Result<BasicResult, Error> {
        let item_id = self.require_item_id(token, id, SearchMode::All)?;
        let url = &format!("{}/api/v1/sec/item/{}/taglink", self.base_url, item_id);
        let token = session::session_token(token, self).context(SessionSnafu)?;
        self.client
            .put(url)
            .header(DOCSPELL_AUTH, token)
            .json(tags)
            .send()
            .and_then(|r| r.error_for_status())
            .context(HttpSnafu { url })?
            .json::<BasicResult>()
            .context(SerializeRespSnafu)
    }

    /// Replaces the given tags on the item with the given id. The id
    /// may be given abbreviated as a prefix, then another request is
    /// used to find the complete id.
    ///
    /// Tags can be given via their names or ids.
    pub fn set_tags<S: AsRef<str>>(
        &self,
        token: &Option<String>,
        id: S,
        tags: &StringList,
    ) -> Result<BasicResult, Error> {
        let item_id = self.require_item_id(token, id, SearchMode::All)?;
        let url = &format!("{}/api/v1/sec/item/{}/tags", self.base_url, item_id);
        let token = session::session_token(token, self).context(SessionSnafu)?;
        self.client
            .put(url)
            .header(DOCSPELL_AUTH, token)
            .json(tags)
            .send()
            .and_then(|r| r.error_for_status())
            .context(HttpSnafu { url })?
            .json::<BasicResult>()
            .context(SerializeRespSnafu)
    }

    /// Removes the given tags on the item with the given id. The id
    /// may be given abbreviated as a prefix, then another request is
    /// used to find the complete id.
    ///
    /// Tags can be given via their names or ids.
    pub fn remove_tags<S: AsRef<str>>(
        &self,
        token: &Option<String>,
        id: S,
        tags: &StringList,
    ) -> Result<BasicResult, Error> {
        let item_id = self.require_item_id(token, id, SearchMode::All)?;
        let url = &format!("{}/api/v1/sec/item/{}/tagsremove", self.base_url, item_id);
        let token = session::session_token(token, self).context(SessionSnafu)?;
        self.client
            .post(url)
            .header(DOCSPELL_AUTH, token)
            .json(tags)
            .send()
            .and_then(|r| r.error_for_status())
            .context(HttpSnafu { url })?
            .json::<BasicResult>()
            .context(SerializeRespSnafu)
    }

    /// Sets the field to the specfified value for the given item.
    pub fn set_field<S: AsRef<str>>(
        &self,
        token: &Option<String>,
        id: S,
        fvalue: &CustomFieldValue,
    ) -> Result<BasicResult, Error> {
        let item_id = self.require_item_id(token, id, SearchMode::All)?;
        let url = &format!("{}/api/v1/sec/item/{}/customfield", self.base_url, item_id);
        let token = session::session_token(token, self).context(SessionSnafu)?;
        self.client
            .put(url)
            .header(DOCSPELL_AUTH, token)
            .json(fvalue)
            .send()
            .and_then(|r| r.error_for_status())
            .context(HttpSnafu { url })?
            .json::<BasicResult>()
            .context(SerializeRespSnafu)
    }

    /// Removes the field from the given item.
    pub fn remove_field<S: AsRef<str>>(
        &self,
        token: &Option<String>,
        id: S,
        field: &str,
    ) -> Result<BasicResult, Error> {
        let item_id = self.require_item_id(token, id, SearchMode::All)?;
        let url = &format!(
            "{}/api/v1/sec/item/{}/customfield/{}",
            self.base_url, item_id, field
        );
        let token = session::session_token(token, self).context(SessionSnafu)?;
        self.client
            .delete(url)
            .header(DOCSPELL_AUTH, token)
            .send()
            .and_then(|r| r.error_for_status())
            .context(HttpSnafu { url })?
            .json::<BasicResult>()
            .context(SerializeRespSnafu)
    }

    /// Given a search query, returns an iterator over all attachments
    /// of the results. The attachments can be downloaded by calling
    /// the corresponding functions on the iterators elements.
    ///
    /// If `token` is specified, it is used to authenticate. Otherwise
    /// a stored session is used.
    pub fn download_search(
        &self,
        token: &Option<String>,
        req: &SearchReq,
    ) -> Result<Downloads, Error> {
        let results = self.search(token, req)?;
        Ok(Downloads::from_results(&results))
    }

    /// Return an iterator over the attachments of the given item.
    /// This just calls `get_item` and returns an iterator over its
    /// attachments.
    pub fn download_attachments<S: AsRef<str>>(
        &self,
        token: &Option<String>,
        id: S,
    ) -> Result<Downloads, Error> {
        let item = self.get_item(token, &id).and_then(|r| {
            r.ok_or(Error::ItemNotFound {
                id: id.as_ref().to_string(),
            })
        })?;
        Ok(Downloads::from_item_detail(&item))
    }

    /// Checks if the integration endpoint is enabled for the given collective.
    pub fn int_endpoint_avail(&self, data: IntegrationData) -> Result<bool, Error> {
        let url = format!(
            "{}/api/v1/open/integration/item/{}",
            self.base_url, data.collective
        );

        let rb = self.client.get(&url);
        let resp = data
            .auth
            .apply(rb)
            .send()
            .context(HttpSnafu { url: url.clone() })?;
        match resp.status() {
            StatusCode::NOT_FOUND => Ok(false),
            StatusCode::UNAUTHORIZED => Err(Error::IntEndpointAuth { url }),
            StatusCode::FORBIDDEN => Err(Error::IntEndpointAuth { url }),
            StatusCode::OK => Ok(true),
            code => {
                resp.error_for_status()
                    .context(HttpSnafu { url: url.clone() })?;
                Err(Error::UnexpectedStatus {
                    status: code.as_u16(),
                    url,
                })
            }
        }
    }

    /// Generates a new invitation key that can be used when
    /// registering an account.
    pub fn gen_invite(&self, req: &GenInvite) -> Result<InviteResult, Error> {
        let url = &format!("{}/api/v1/open/signup/newinvite", self.base_url);
        self.client
            .post(url)
            .json(req)
            .send()
            .context(HttpSnafu { url })?
            .json::<InviteResult>()
            .context(SerializeRespSnafu)
    }

    /// Registers a new account with Docspell.
    pub fn register(&self, req: &Registration) -> Result<BasicResult, Error> {
        let url = &format!("{}/api/v1/open/signup/register", self.base_url);
        log::debug!("Register new account: {:?}", req);
        self.client
            .post(url)
            .json(req)
            .send()
            .and_then(|r| r.error_for_status())
            .context(HttpSnafu { url })?
            .json::<BasicResult>()
            .context(SerializeRespSnafu)
    }

    /// Checks if a file with give hash (sha256) exists in Docspell.
    ///
    /// Authentication can be via the session, a source id or the
    /// integration endpoint as defined via the `FileAuth`.
    pub fn file_exists<S: Into<String>>(
        &self,
        hash: S,
        file_auth: &FileAuth,
    ) -> Result<CheckFileResult, Error> {
        let url = match file_auth {
            FileAuth::Source { id } => {
                format!(
                    "{}/api/v1/open/checkfile/{}/{}",
                    self.base_url,
                    id,
                    hash.into()
                )
            }
            FileAuth::Integration(IntegrationData { collective, .. }) => format!(
                "{}/api/v1/open/integration/checkfile/{}/{}",
                self.base_url,
                collective,
                hash.into()
            ),
            FileAuth::Session { .. } => {
                format!("{}/api/v1/sec/checkfile/{}", self.base_url, hash.into())
            }
        };

        let rb = self.client.get(&url);
        file_auth
            .apply(self, rb)?
            .send()
            .and_then(|r| r.error_for_status())
            .context(HttpSnafu { url })?
            .json::<CheckFileResult>()
            .context(SerializeRespSnafu)
    }

    /// Upload some files for processing.
    ///
    /// The `meta` part can be used to control some parts of
    /// processing. The vec of files must not be empty.
    ///
    /// Authentication can be via the session, a source id or the
    /// integration endpoint as defined via the `FileAuth`.
    pub fn upload_files(
        &self,
        file_auth: &FileAuth,
        meta: &UploadMeta,
        files: &[&Path],
    ) -> Result<BasicResult, Error> {
        let url = match file_auth {
            FileAuth::Source { id } => {
                format!("{}/api/v1/open/upload/item/{}", self.base_url, id,)
            }
            FileAuth::Integration(IntegrationData { collective, .. }) => format!(
                "{}/api/v1/open/integration/item/{}",
                self.base_url, collective,
            ),
            FileAuth::Session { .. } => {
                format!("{}/api/v1/sec/upload/item", self.base_url)
            }
        };

        let meta_json = serde_json::to_vec(&meta).context(SerializeReqSnafu)?;
        let meta_part = Part::bytes(meta_json)
            .mime_str(APP_JSON)
            .context(MimeSnafu { raw: APP_JSON })?;
        let mut form = Form::new().part("meta", meta_part);
        for path in files {
            log::debug!("Adding to request: {}", path.display());

            let fopen = File::open(path).context(OpenFileSnafu { path })?;
            let len = fopen.metadata().context(OpenFileSnafu { path })?.len();
            let bufr = std::io::BufReader::new(fopen);
            let mut fpart = Part::reader_with_length(bufr, len);
            if let Some(fname) = path.file_name() {
                let f: String = fname.to_string_lossy().into();
                fpart = fpart.file_name(f);
            }
            form = form.part("file", fpart);
        }

        file_auth
            .apply(self, self.client.post(&url))?
            .multipart(form)
            .send()
            .and_then(|r| r.error_for_status())
            .context(HttpSnafu { url })?
            .json::<BasicResult>()
            .context(SerializeRespSnafu)
    }

    /// Submits a task on the Docspell server, that (re)generates all preview images.
    ///
    /// This is needed if the preview dpi setting has been changed.
    ///
    /// It requires to provide the admin secret from Docspells configuration file.
    pub fn admin_generate_previews<S: Into<String>>(
        &self,
        admin_secret: S,
    ) -> Result<BasicResult, Error> {
        let url = &format!(
            "{}/api/v1/admin/attachments/generatePreviews",
            self.base_url
        );
        self.client
            .post(url)
            .header(DOCSPELL_ADMIN, admin_secret.into())
            .send()
            .and_then(|r| r.error_for_status())
            .context(HttpSnafu { url })?
            .json::<BasicResult>()
            .context(SerializeRespSnafu)
    }

    /// Submits a task to re-create the entire fulltext index (across all collectives).
    ///
    /// It requires to provide the admin secret from Docspells configuration file.
    pub fn admin_recreate_index<S: Into<String>>(
        &self,
        admin_secret: S,
    ) -> Result<BasicResult, Error> {
        let url = &format!("{}/api/v1/admin/fts/reIndexAll", self.base_url);
        self.client
            .post(url)
            .header(DOCSPELL_ADMIN, admin_secret.into())
            .send()
            .and_then(|r| r.error_for_status())
            .context(HttpSnafu { url })?
            .json::<BasicResult>()
            .context(SerializeRespSnafu)
    }

    /// Submits a task to convert all (not yet converted) pdfs via the
    /// configured tool (by default ocrmypdf).
    ///
    /// It requires to provide the admin secret from Docspells
    /// configuration file.
    pub fn admin_convert_all_pdfs<S: Into<String>>(
        &self,
        admin_secret: S,
    ) -> Result<BasicResult, Error> {
        let url = &format!("{}/api/v1/admin/attachments/convertallpdfs", self.base_url);
        self.client
            .post(url)
            .header(DOCSPELL_ADMIN, admin_secret.into())
            .send()
            .and_then(|r| r.error_for_status())
            .context(HttpSnafu { url })?
            .json::<BasicResult>()
            .context(SerializeRespSnafu)
    }

    /// Resets the password for the given account.
    ///
    /// It requires to provide the admin secret from Docspells configuration file.
    pub fn admin_reset_password<S: Into<String>>(
        &self,
        admin_secret: S,
        account: &Account,
    ) -> Result<ResetPasswordResp, Error> {
        let url = &format!("{}/api/v1/admin/user/resetPassword", self.base_url);
        self.client
            .post(url)
            .header(DOCSPELL_ADMIN, admin_secret.into())
            .json(&account)
            .send()
            .and_then(|r| r.error_for_status())
            .context(HttpSnafu { url })?
            .json::<ResetPasswordResp>()
            .context(SerializeRespSnafu)
    }

    pub fn admin_reset_otp<S: Into<String>>(
        &self,
        admin_secret: S,
        account: &Account,
    ) -> Result<BasicResult, Error> {
        let url = &format!("{}/api/v1/admin/user/otp/resetOTP", self.base_url);
        self.client
            .post(url)
            .header(DOCSPELL_ADMIN, admin_secret.into())
            .json(&account)
            .send()
            .and_then(|r| r.error_for_status())
            .context(HttpSnafu { url })?
            .json::<BasicResult>()
            .context(SerializeRespSnafu)
    }

    // --- Helpers

    fn require_item_id<S: AsRef<str>>(
        &self,
        token: &Option<String>,
        partial_id: S,
        search_mode: SearchMode,
    ) -> Result<String, Error> {
        let id_s: &str = partial_id.as_ref();
        self.complete_item_id(token, id_s, search_mode)?
            .ok_or(Error::ItemNotFound {
                id: id_s.to_string(),
            })
    }

    /// Search for a unique item given a partial id.
    fn complete_item_id(
        &self,
        token: &Option<String>,
        partial_id: &str,
        search_mode: SearchMode,
    ) -> Result<Option<String>, Error> {
        if partial_id.len() < ID_LEN {
            log::debug!(
                "Item id '{}' is not complete, searching for the item via a query",
                partial_id
            );
            let req = SearchReq {
                offset: 0,
                limit: 2,
                with_details: false,
                query: format!("id:{}*", partial_id),
                search_mode,
            };
            self.search(token, &req)
                .and_then(|r| Self::find_id(partial_id, &r))
        } else {
            Ok(Some(partial_id.into()))
        }
    }

    /// Find the single item id or return an error
    fn find_id(id: &str, results: &SearchResult) -> Result<Option<String>, Error> {
        match results.groups.len() {
            0 => Ok(None),
            1 => match results.groups[0].items.len() {
                0 => Ok(None),
                1 => Ok(Some(results.groups[0].items[0].id.clone())),
                _ => Err(Error::ItemNotUnique { id: id.into() }),
            },
            _ => Err(Error::ItemNotUnique { id: id.into() }),
        }
    }
}

/// Defines methods to authenticate when uploading files.
///
/// Either use a [source
/// id](https://docspell.org/docs/webapp/uploading/#anonymous-upload),
/// use the [integration
/// endpoint](https://docspell.org/docs/api/upload/#integration-endpoint)
/// or the session.
pub enum FileAuth {
    Source { id: String },
    Integration(IntegrationData),
    Session { token: Option<String> },
}

/// When using the integration endpoint, a collective id is required
/// and possibly some authentication information.
pub struct IntegrationData {
    pub collective: String,
    pub auth: IntegrationAuth,
}

/// The integration endpoint allows several authentication methods:
/// via http basic, some other specific header or without any extra
/// data (using fixed ip addresses).
pub enum IntegrationAuth {
    Header(String, String),
    Basic(String, String),
    None,
}
impl IntegrationAuth {
    fn apply(&self, rb: RequestBuilder) -> RequestBuilder {
        match self {
            IntegrationAuth::Header(name, value) => {
                log::debug!("Using integration endpoint with header: {}:{}", name, value);
                rb.header(name, value)
            }
            IntegrationAuth::Basic(name, pass) => {
                log::debug!("Using integration endpoint with basic auth: {}:***", name,);
                rb.basic_auth(name, Some(pass))
            }
            IntegrationAuth::None => rb,
        }
    }
}

impl FileAuth {
    pub fn from_session<S: Into<String>>(token: S) -> FileAuth {
        FileAuth::Session {
            token: Some(token.into()),
        }
    }

    pub fn from_source<S: Into<String>>(source_id: S) -> FileAuth {
        FileAuth::Source {
            id: source_id.into(),
        }
    }

    fn apply(&self, client: &Client, rb: RequestBuilder) -> Result<RequestBuilder, Error> {
        match self {
            FileAuth::Source { .. } => Ok(rb),
            FileAuth::Integration(IntegrationData { auth, .. }) => Ok(auth.apply(rb)),
            FileAuth::Session { token } => {
                let h = session::session_token(token, client).context(SessionSnafu)?;
                Ok(rb.header(DOCSPELL_AUTH, h))
            }
        }
    }
}

/// Represents a response to a request to a binary file.
///
/// The methods allow to retrieve the filename from the response or
/// copy the bytes somewhere.
pub struct Download {
    pub id: String,
    pub url: String,
    pub name: String,
    resp: Response,
}

impl Download {
    /// Get the filename from the responses `Content-Disposition`
    /// header.
    pub fn get_filename(&self) -> Option<&str> {
        self.resp
            .headers()
            .get("Content-Disposition")
            .and_then(|hv| hv.to_str().ok())
            .and_then(util::filename_from_header)
    }

    /// Copies the bytes from the response into the give writer.
    pub fn copy_to<W: ?Sized>(&mut self, w: &mut W) -> Result<u64, Error>
    where
        W: Write,
    {
        let resp = &mut self.resp;
        resp.copy_to(w).context(HttpSnafu {
            url: self.url.clone(),
        })
    }
}

/// A reference to an attachment.
///
/// It contains its id and name. With this information, use one of its
/// methods to download the desired file.
pub struct DownloadRef {
    pub id: String,
    pub name: String,
}
impl DownloadRef {
    fn from(idname: IdName) -> DownloadRef {
        DownloadRef::new(idname.id, idname.name)
    }

    fn new<S: Into<String>>(id: S, name: S) -> DownloadRef {
        DownloadRef {
            id: id.into(),
            name: name.into(),
        }
    }

    pub fn has_archive(&self, client: &Client, token: &Option<String>) -> Result<bool, Error> {
        let url = format!(
            "{}/api/v1/sec/attachment/{}/archive",
            client.base_url, self.id
        );
        self.head_file(client, token, &url)
    }

    pub fn get(&self, client: &Client, token: &Option<String>) -> Result<Option<Download>, Error> {
        let url = format!("{}/api/v1/sec/attachment/{}", client.base_url, self.id);
        self.get_file(client, token, &url)
    }

    pub fn get_original(
        &self,
        client: &Client,
        token: &Option<String>,
    ) -> Result<Option<Download>, Error> {
        let url = format!(
            "{}/api/v1/sec/attachment/{}/original",
            client.base_url, self.id
        );
        self.get_file(client, token, &url)
    }

    pub fn get_archive(
        &self,
        client: &Client,
        token: &Option<String>,
    ) -> Result<Option<Download>, Error> {
        let url = format!(
            "{}/api/v1/sec/attachment/{}/archive",
            client.base_url, self.id
        );
        self.get_file(client, token, &url)
    }

    fn get_file(
        &self,
        client: &Client,
        token: &Option<String>,
        url: &str,
    ) -> Result<Option<Download>, Error> {
        let token = session::session_token(token, client).context(SessionSnafu)?;
        let resp = client
            .client
            .get(url)
            .header(DOCSPELL_AUTH, &token)
            .send()
            .context(HttpSnafu { url })?;
        if resp.status() == StatusCode::NOT_FOUND {
            Ok(None)
        } else {
            Ok(Some(Download {
                id: self.id.clone(),
                url: url.to_string(),
                resp: resp.error_for_status().context(HttpSnafu { url })?,
                name: self.name.clone(),
            }))
        }
    }

    fn head_file(&self, client: &Client, token: &Option<String>, url: &str) -> Result<bool, Error> {
        let token = session::session_token(token, client).context(SessionSnafu)?;
        let resp = client
            .client
            .head(url)
            .header(DOCSPELL_AUTH, &token)
            .send()
            .context(HttpSnafu { url })?;
        if resp.status() == StatusCode::NOT_FOUND {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// An iterator over `DownloadRef` elements.
pub struct Downloads {
    refs: Vec<DownloadRef>,
}

impl Downloads {
    pub fn from_item(item: &Item) -> Downloads {
        let refs: Vec<DownloadRef> = item
            .attachments
            .iter()
            .map(|a| DownloadRef::from(a.to_idname()))
            .collect();
        Downloads { refs }
    }

    pub fn from_item_detail(item: &ItemDetail) -> Downloads {
        let refs: Vec<DownloadRef> = item
            .attachments
            .iter()
            .map(|a| DownloadRef::from(a.to_idname()))
            .collect();
        Downloads { refs }
    }

    pub fn from_results(results: &SearchResult) -> Downloads {
        let refs: Vec<DownloadRef> = results
            .groups
            .iter()
            .flat_map(|g| g.items.iter())
            .flat_map(|i| i.attachments.iter())
            .map(|a| DownloadRef::from(a.to_idname()))
            .collect();

        Downloads { refs }
    }

    pub fn is_empty(&self) -> bool {
        self.refs.is_empty()
    }

    pub fn non_empty(&self) -> bool {
        !self.is_empty()
    }

    pub fn len(&self) -> usize {
        self.refs.len()
    }
}
impl Iterator for Downloads {
    type Item = DownloadRef;

    fn next(&mut self) -> Option<DownloadRef> {
        self.refs.pop()
    }
}
