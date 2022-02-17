//! Defines all commands of the cli.
//!
//! A command is defined by the trait [`Cmd`]. Besides the type it is
//! defined on, it expects a [`Context`] argument which contains the
//! configuration file, the common options and an instance of the
//! [`crate::http::Client`].
//!
//! Each command defines its inputs via [clap](https://clap.rs) and
//! implements for this type the `Cmd` trait. Each input type is
//! referenced in the subcommand enum.

pub mod admin;
pub mod cleanup;
pub mod download;
pub mod export;
pub mod file_exists;
pub mod generate_completions;
pub mod geninvite;
pub mod item;
pub mod login;
pub mod logout;
pub mod register;
pub mod search;
pub mod search_summary;
pub mod source;
pub mod upload;
pub mod version;
pub mod view;
pub mod watch;

use super::opts::Format;
use super::sink::{Error as SinkError, Sink};
use crate::cli::opts::CommonOpts;
use crate::config::{ConfigError, DsConfig};
use crate::http::{self, Client};
use serde::Serialize;
use snafu::{ResultExt, Snafu};

/// A command for the cli.
///
/// The [`Context`] argument is defined for all commands.
pub trait Cmd {
    type CmdError;

    fn exec<'a>(&self, args: &'a Context) -> Result<(), Self::CmdError>;
}

/// An environment for running a command.
///
/// It has by default access to the configuration and the common
/// options. The http client is also provided.
pub struct Context<'a> {
    pub opts: &'a CommonOpts,
    pub cfg: &'a DsConfig,
    pub client: Client,
}

impl Context<'_> {
    pub fn new<'a>(opts: &'a CommonOpts, cfg: &'a DsConfig) -> Result<Context<'a>, CmdError> {
        let client = Client::new(docspell_url(opts, cfg), opts.to_proxy_setting())
            .context(ContextCreateSnafu)?;
        Ok(Context { opts, cfg, client })
    }

    fn base_url(&self) -> String {
        docspell_url(self.opts, self.cfg)
    }

    fn write_result<A: Sink + Serialize>(&self, value: A) -> Result<(), SinkError> {
        let fmt = self.format();
        Sink::write_value(fmt, &value)
    }

    fn format(&self) -> Format {
        self.opts.format.unwrap_or(self.cfg.default_format)
    }
}

fn docspell_url(opts: &CommonOpts, cfg: &DsConfig) -> String {
    opts.docspell_url
        .as_ref()
        .unwrap_or(&cfg.docspell_url)
        .clone()
}

#[derive(Debug, Snafu)]
pub enum CmdError {
    ContextCreate { source: http::Error },
    Export { source: export::Error },
    Watch { source: watch::Error },
    Upload { source: upload::Error },
    Admin { source: admin::Error },
    Cleanup { source: cleanup::Error },
    Download { source: download::Error },
    FileExists { source: file_exists::Error },
    GenInvite { source: geninvite::Error },
    Item { source: item::Error },
    Login { source: login::Error },
    Logout { source: logout::Error },
    Register { source: register::Error },
    Search { source: search::Error },
    SearchSummary { source: search_summary::Error },
    Source { source: source::Error },
    Version { source: version::Error },
    View { source: view::Error },
    WriteConfig { source: ConfigError },
    WriteSink { source: SinkError },
}

impl From<ConfigError> for CmdError {
    fn from(source: ConfigError) -> Self {
        CmdError::WriteConfig { source }
    }
}
impl From<version::Error> for CmdError {
    fn from(source: version::Error) -> Self {
        CmdError::Version { source }
    }
}
impl From<login::Error> for CmdError {
    fn from(source: login::Error) -> Self {
        CmdError::Login { source }
    }
}
impl From<logout::Error> for CmdError {
    fn from(source: logout::Error) -> Self {
        CmdError::Logout { source }
    }
}
impl From<search::Error> for CmdError {
    fn from(source: search::Error) -> Self {
        CmdError::Search { source }
    }
}
impl From<file_exists::Error> for CmdError {
    fn from(source: file_exists::Error) -> Self {
        CmdError::FileExists { source }
    }
}
impl From<geninvite::Error> for CmdError {
    fn from(source: geninvite::Error) -> Self {
        CmdError::GenInvite { source }
    }
}
impl From<register::Error> for CmdError {
    fn from(source: register::Error) -> Self {
        CmdError::Register { source }
    }
}
impl From<search_summary::Error> for CmdError {
    fn from(source: search_summary::Error) -> Self {
        CmdError::SearchSummary { source }
    }
}
impl From<source::Error> for CmdError {
    fn from(source: source::Error) -> Self {
        CmdError::Source { source }
    }
}
impl From<item::Error> for CmdError {
    fn from(source: item::Error) -> Self {
        CmdError::Item { source }
    }
}
impl From<admin::Error> for CmdError {
    fn from(source: admin::Error) -> Self {
        CmdError::Admin { source }
    }
}
impl From<download::Error> for CmdError {
    fn from(source: download::Error) -> Self {
        CmdError::Download { source }
    }
}
impl From<view::Error> for CmdError {
    fn from(source: view::Error) -> Self {
        CmdError::View { source }
    }
}
impl From<cleanup::Error> for CmdError {
    fn from(source: cleanup::Error) -> Self {
        CmdError::Cleanup { source }
    }
}
impl From<upload::Error> for CmdError {
    fn from(source: upload::Error) -> Self {
        CmdError::Upload { source }
    }
}
impl From<watch::Error> for CmdError {
    fn from(source: watch::Error) -> Self {
        CmdError::Watch { source }
    }
}
impl From<export::Error> for CmdError {
    fn from(source: export::Error) -> Self {
        CmdError::Export { source }
    }
}
