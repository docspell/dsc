pub mod payload;
pub mod session;
mod util;

use self::payload::*;
use self::util::{DOCSPELL_ADMIN, DOCSPELL_AUTH};
use reqwest::{blocking::RequestBuilder, StatusCode};
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An error was received from: {}!", url))]
    Http { source: reqwest::Error, url: String },

    #[snafu(display("Session error: {}", source))]
    Session { source: self::session::Error },

    #[snafu(display("An error occured serializing the response!"))]
    SerializeResp { source: reqwest::Error },

    #[snafu(display("Login failed!"))]
    LoginFailed,

    #[snafu(display("Authentication failure for integration endpoint: {}", url))]
    IntEndpointAuth { url: String },

    #[snafu(display("Unexpected response status: {}", status))]
    UnexpectedStatus { status: u16, url: String },
}

pub struct Client {
    client: reqwest::blocking::Client,
    base_url: String,
}

impl Client {
    pub fn new<S: Into<String>>(docspell_url: S) -> Client {
        Client {
            client: reqwest::blocking::Client::new(),
            base_url: docspell_url.into(),
        }
    }

    pub fn version(&self) -> Result<VersionInfo, Error> {
        let url = &format!("{}/api/info/version", self.base_url);
        return self
            .client
            .get(url)
            .send()
            .context(Http { url })?
            .json::<VersionInfo>()
            .context(SerializeResp);
    }

    pub fn login(&self, req: &AuthRequest) -> Result<AuthResp, Error> {
        let url = &format!("{}/api/v1/open/auth/login", self.base_url);
        let result = self
            .client
            .post(url)
            .json(req)
            .send()
            .and_then(|r| r.error_for_status())
            .context(Http { url })?
            .json::<AuthResp>()
            .context(SerializeResp)?;

        if result.success {
            session::store_session(&result).context(Session)?;
            Ok(result)
        } else {
            log::debug!("Login result: {:?}", result);
            Err(Error::LoginFailed)
        }
    }

    pub fn logout(&self) -> Result<(), Error> {
        session::drop_session().context(Session)
    }

    pub fn session_login(&self, token: &str) -> Result<AuthResp, Error> {
        let url = &format!("{}/api/v1/sec/auth/session", self.base_url);
        let result = self
            .client
            .post(url)
            .header(DOCSPELL_AUTH, token)
            .send()
            .and_then(|r| r.error_for_status())
            .context(Http { url })?
            .json::<AuthResp>()
            .context(SerializeResp)?;

        if result.success {
            Ok(result)
        } else {
            log::debug!("Session login result: {:?}", result);
            Err(Error::LoginFailed)
        }
    }

    pub fn search(&self, token: &Option<String>, req: &SearchReq) -> Result<SearchResult, Error> {
        let url = &format!("{}/api/v1/sec/item/search", self.base_url);
        let token = session::session_token(token, self).context(Session)?;
        self.client
            .get(url)
            .header(DOCSPELL_AUTH, token)
            .query(&[
                ("limit", &req.limit.to_string()),
                ("offset", &req.offset.to_string()),
                ("withDetails", &req.with_details.to_string()),
                ("q", &req.query),
            ])
            .send()
            .and_then(|r| r.error_for_status())
            .context(Http { url })?
            .json::<SearchResult>()
            .context(SerializeResp)
    }

    pub fn summary<S: Into<String>>(
        &self,
        token: &Option<String>,
        query: S,
    ) -> Result<Summary, Error> {
        let url = &format!("{}/api/v1/sec/item/searchStats", self.base_url);
        let token = session::session_token(token, self).context(Session)?;
        self.client
            .get(url)
            .header(DOCSPELL_AUTH, token)
            .query(&[("q", &query.into())])
            .send()
            .and_then(|r| r.error_for_status())
            .context(Http { url })?
            .json::<Summary>()
            .context(SerializeResp)
    }

    pub fn list_sources(&self, token: &Option<String>) -> Result<SourceList, Error> {
        let url = &format!("{}/api/v1/sec/source", self.base_url);
        let token = session::session_token(token, self).context(Session)?;
        self.client
            .get(url)
            .header(DOCSPELL_AUTH, token)
            .send()
            .and_then(|r| r.error_for_status())
            .context(Http { url })?
            .json::<SourceList>()
            .context(SerializeResp)
    }

    pub fn get_item<S: Into<String>>(
        &self,
        token: &Option<String>,
        id: S,
    ) -> Result<Option<ItemDetail>, Error> {
        let url = &format!("{}/api/v1/sec/item/{}", self.base_url, id.into());
        let token = session::session_token(token, self).context(Session)?;
        let resp = self
            .client
            .get(url)
            .header(DOCSPELL_AUTH, token)
            .send()
            .context(Http { url })?;

        if resp.status() == StatusCode::NOT_FOUND {
            Ok(None)
        } else {
            resp.error_for_status()
                .context(Http { url })?
                .json::<ItemDetail>()
                .context(SerializeResp)
                .map(|r| Some(r))
        }
    }

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
            .context(Http { url: url.clone() })?;
        match resp.status() {
            StatusCode::NOT_FOUND => Ok(false),
            StatusCode::UNAUTHORIZED => Err(Error::IntEndpointAuth { url }),
            StatusCode::FORBIDDEN => Err(Error::IntEndpointAuth { url }),
            StatusCode::OK => Ok(true),
            code => {
                resp.error_for_status().context(Http { url: url.clone() })?;
                Err(Error::UnexpectedStatus {
                    status: code.as_u16(),
                    url: url.clone(),
                })
            }
        }
    }

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
            .context(Http { url })?
            .json::<CheckFileResult>()
            .context(SerializeResp)
    }

    pub fn gen_invite(&self, req: &GenInvite) -> Result<InviteResult, Error> {
        let url = &format!("{}/api/v1/open/signup/newinvite", self.base_url);
        self.client
            .post(url)
            .json(req)
            .send()
            .context(Http { url })?
            .json::<InviteResult>()
            .context(SerializeResp)
    }

    pub fn register(&self, req: &Registration) -> Result<BasicResult, Error> {
        let url = &format!("{}/api/v1/open/signup/register", self.base_url);
        log::debug!("Register new account: {:?}", req);
        self.client
            .post(url)
            .json(req)
            .send()
            .and_then(|r| r.error_for_status())
            .context(Http { url })?
            .json::<BasicResult>()
            .context(SerializeResp)
    }

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
            .context(Http { url })?
            .json::<BasicResult>()
            .context(SerializeResp)
    }

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
            .context(Http { url })?
            .json::<BasicResult>()
            .context(SerializeResp)
    }

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
            .context(Http { url })?
            .json::<ResetPasswordResp>()
            .context(SerializeResp)
    }
}

pub enum FileAuth {
    Source { id: String },
    Integration(IntegrationData),
    Session { token: Option<String> },
}

pub struct IntegrationData {
    pub collective: String,
    pub auth: IntegrationAuth,
}

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
                let h = session::session_token(token, client).context(Session)?;
                Ok(rb.header(DOCSPELL_AUTH, h))
            }
        }
    }
}
