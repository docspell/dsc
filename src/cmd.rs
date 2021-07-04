pub mod version;

use crate::config::DsConfig;
use serde::Serialize;

pub trait Cmd {
    fn exec(&self, cfg: DsConfig) -> Result<(), CmdError>;

    fn json_str<A: Serialize>(arg: &A) -> Result<String, CmdError> {
        serde_json::to_string(arg).map_err(CmdError::JsonSerError)
    }
    fn sexpr_str<A: Serialize>(arg: &A) -> Result<String, CmdError> {
        serde_lexpr::to_string(arg).map_err(CmdError::SexprError)
    }
}

#[derive(Debug)]
pub enum CmdError {
    HttpError(reqwest::Error),
    JsonSerError(serde_json::Error),
    SexprError(serde_lexpr::Error),
}
