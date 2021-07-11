pub mod admin;
pub mod file_exists;
pub mod geninvite;
pub mod login;
pub mod register;
pub mod search;
pub mod search_summary;
pub mod source;
pub mod upload;
pub mod version;

use crate::{
    config::DsConfig,
    opts::{CommonOpts, Format},
    sink::{SerError, Sink},
};
use serde::Serialize;

pub trait Cmd {
    fn exec<'a>(&self, args: &'a CmdArgs) -> Result<(), CmdError>;
}

pub struct CmdArgs<'a> {
    pub opts: &'a CommonOpts,
    pub cfg: &'a DsConfig,
}

impl CmdArgs<'_> {
    fn write_result<A: Sink + Serialize>(&self, value: A) -> Result<(), CmdError> {
        let fmt = self.format();
        Sink::write_value(fmt, &value)?;
        Ok(())
    }

    fn format(&self) -> Format {
        self.opts.format.unwrap_or(self.cfg.default_format)
    }

    fn docspell_url(&self) -> String {
        self.opts
            .docspell_url
            .as_ref()
            .unwrap_or(&self.cfg.docspell_url)
            .clone()
    }

    fn admin_secret(&self) -> Option<String> {
        self.opts
            .admin_secret
            .as_ref()
            .or(self.cfg.admin_secret.as_ref())
            .map(String::clone)
    }

    fn pass_entry(&self, given: &Option<String>) -> Option<String> {
        given.clone().or(self.cfg.pass_entry.clone())
    }
}

#[derive(Debug)]
pub enum CmdError {
    HttpError(reqwest::Error),
    SerializeError(SerError),
    JsonSerError(serde_json::Error),
    SexprError(serde_lexpr::Error),
    AuthError(String),
    IOError(std::io::Error),
    InvalidInput(String),
    IntEndpointNotAvail,
}

impl std::convert::From<serde_json::Error> for CmdError {
    fn from(e: serde_json::Error) -> CmdError {
        CmdError::JsonSerError(e)
    }
}
impl std::convert::From<serde_lexpr::Error> for CmdError {
    fn from(e: serde_lexpr::Error) -> CmdError {
        CmdError::SexprError(e)
    }
}
impl std::convert::From<reqwest::Error> for CmdError {
    fn from(e: reqwest::Error) -> CmdError {
        CmdError::HttpError(e)
    }
}
impl std::convert::From<std::io::Error> for CmdError {
    fn from(e: std::io::Error) -> CmdError {
        CmdError::IOError(e)
    }
}
impl std::convert::From<SerError> for CmdError {
    fn from(e: SerError) -> CmdError {
        CmdError::SerializeError(e)
    }
}
