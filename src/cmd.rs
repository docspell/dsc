pub mod version;

use crate::config::DsConfig;
use crate::opts::{CommonOpts, Format};
use serde::Serialize;

pub trait Cmd {
    fn exec<'a>(&self, args: &'a CmdArgs) -> Result<(), CmdError>;
}

pub struct CmdArgs<'a> {
    pub cfg: &'a DsConfig,
    pub opts: &'a CommonOpts,
}

impl CmdArgs<'_> {
    fn make_str<A: Serialize>(&self, arg: &A) -> Result<String, CmdError> {
        let fmt = self.opts.format.unwrap_or(self.cfg.default_format);
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
}
