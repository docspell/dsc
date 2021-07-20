pub mod payload;
pub mod session;
mod util;

use self::payload::*;
use self::util::DOCSPELL_AUTH;
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
}
