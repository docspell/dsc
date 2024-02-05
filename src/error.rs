//! Global error types.

use crate::cli::cmd;
use crate::config;
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{}", source))]
    Cmd { source: cmd::CmdError },

    #[snafu(display("Configuration error: {}", source))]
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
