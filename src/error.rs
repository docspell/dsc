use crate::cmd;
use crate::config;
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
    Cmd { source: cmd::CmdError },
    Config { source: config::ConfigError },
}

impl From<config::ConfigError> for Error {
    fn from(e: config::ConfigError) -> Error {
        Error::Config { source: e }
    }
}

impl From<cmd::CmdError> for Error {
    fn from(e: cmd::CmdError) -> Error {
        Error::Cmd { source: e }
    }
}

pub type Result<A> = std::result::Result<A, Error>;
