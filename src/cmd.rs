pub mod version;

use crate::config::DsConfig;
use crate::opts::Format;
use serde::Serialize;

pub trait Cmd {
    fn exec(&self, cfg: &DsConfig) -> Result<(), CmdError>;

    fn make_str<A: Serialize>(format: Option<&Format>, arg: &A) -> Result<String, CmdError> {
        let fmt = format.unwrap_or(&Format::Json);
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
