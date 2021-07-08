pub mod admin;
pub mod file_exists;
pub mod login;
pub mod search;
pub mod search_summary;
pub mod source;
pub mod version;

use crate::opts::{ConfigOpts, Format};
use serde::Serialize;

pub trait Cmd {
    fn exec<'a>(&self, args: &'a CmdArgs) -> Result<(), CmdError>;
}

pub struct CmdArgs<'a> {
    pub opts: &'a ConfigOpts,
}

impl CmdArgs<'_> {
    fn make_str<A: Serialize>(&self, arg: &A) -> Result<String, CmdError> {
        let fmt = self.opts.format;
        match fmt {
            Format::Json => serde_json::to_string(arg).map_err(CmdError::JsonSerError),
            Format::Lisp => serde_lexpr::to_string(arg).map_err(CmdError::SexprError),
        }
    }
}

#[derive(Debug)]
pub enum CmdError {
    HttpError(reqwest::Error),
    JsonSerError(serde_json::Error),
    SexprError(serde_lexpr::Error),
    AuthError(String),
    IOError(std::io::Error),
    InvalidInput(String),
    IntEndpointNotAvail,
}
