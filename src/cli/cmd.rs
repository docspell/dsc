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
pub mod bookmark;
pub mod cleanup;
pub mod download;
pub mod export;
pub mod file_exists;
pub mod generate_completions;
pub mod geninvite;
pub mod item;
pub mod login;
pub mod logout;
pub mod open_item;
pub mod register;
pub mod search;
pub mod search_summary;
pub mod source;
pub mod upload;
pub mod version;
pub mod view;
pub mod watch;

use std::path::PathBuf;
use std::str::FromStr;

use super::opts::Format;
use super::sink::{Error as SinkError, Sink};
use crate::cli;
use crate::cli::opts::CommonOpts;
use crate::config::{ConfigError, DsConfig};
use crate::http::proxy::ProxySetting;
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
        let client = Client::new(
            docspell_url(opts, cfg),
            proxy_settings(opts, cfg),
            &extra_certificate(opts, cfg),
            accept_invalid_certs(opts, cfg),
        )
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
    match &opts.docspell_url {
        Some(u) => {
            log::debug!("Use docspell url from arguments: {}", u);
            u.clone()
        }
        None => match std::env::var(DSC_DOCSPELL_URL).ok() {
            Some(u) => {
                log::debug!("Use docspell url from env: {}", u);
                u
            }
            None => {
                log::debug!("Use docspell url from config: {}", cfg.docspell_url);
                cfg.docspell_url.clone()
            }
        },
    }
}

fn accept_invalid_certs(opts: &CommonOpts, cfg: &DsConfig) -> bool {
    opts.accept_invalid_certificates || cfg.accept_invalid_certificates.unwrap_or(false)
}

fn extra_certificate(opts: &CommonOpts, cfg: &DsConfig) -> Option<PathBuf> {
    opts.extra_certificate
        .clone()
        .or_else(|| cfg.extra_certificate.clone())
}

fn proxy_settings(opts: &CommonOpts, cfg: &DsConfig) -> ProxySetting {
    let user = opts.proxy_user.clone().or_else(|| cfg.proxy_user.clone());
    let pass = opts
        .proxy_password
        .clone()
        .or_else(|| cfg.proxy_password.clone());
    let prx = opts.proxy.clone().or_else(|| match &cfg.proxy {
        None => None,
        Some(str) => cli::opts::ProxySetting::from_str(str).ok(),
    });

    log::debug!("Using proxy: {:?} @ {:?}", user, prx);
    CommonOpts::to_proxy_setting(&prx, user, pass)
}

#[derive(Debug, Snafu)]
pub enum CmdError {
    #[snafu(display("Bookmark - {}", source))]
    Bookmark { source: bookmark::Error },

    #[snafu(display("ContextCreate - {}", source))]
    ContextCreate { source: http::Error },

    #[snafu(display("Export - {}", source))]
    Export { source: export::Error },

    #[snafu(display("Watch - {}", source))]
    Watch { source: watch::Error },

    #[snafu(display("Upload - {}", source))]
    Upload { source: upload::Error },

    #[snafu(display("Admin - {}", source))]
    Admin { source: admin::Error },

    #[snafu(display("Cleanup - {}", source))]
    Cleanup { source: cleanup::Error },

    #[snafu(display("Download - {}", source))]
    Download { source: download::Error },

    #[snafu(display("FileExists - {}", source))]
    FileExists { source: file_exists::Error },

    #[snafu(display("GenInvite - {}", source))]
    GenInvite { source: geninvite::Error },

    #[snafu(display("Item - {}", source))]
    Item { source: item::Error },

    #[snafu(display("Login - {}", source))]
    Login { source: login::Error },

    #[snafu(display("Logout - {}", source))]
    Logout { source: logout::Error },

    #[snafu(display("OpenItem - {}", source))]
    OpenItem { source: open_item::Error },

    #[snafu(display("Register - {}", source))]
    Register { source: register::Error },

    #[snafu(display("Search - {}", source))]
    Search { source: search::Error },

    #[snafu(display("SearchSummary - {}", source))]
    SearchSummary { source: search_summary::Error },

    #[snafu(display("Source - {}", source))]
    Source { source: source::Error },

    #[snafu(display("Version - {}", source))]
    Version { source: version::Error },

    #[snafu(display("View - {}", source))]
    View { source: view::Error },

    #[snafu(display("WriteConfig - {}", source))]
    WriteConfig { source: ConfigError },

    #[snafu(display("{}", source))]
    WriteSink { source: SinkError },
}

impl From<bookmark::Error> for CmdError {
    fn from(source: bookmark::Error) -> Self {
        CmdError::Bookmark { source }
    }
}

impl From<open_item::Error> for CmdError {
    fn from(source: open_item::Error) -> Self {
        CmdError::OpenItem { source }
    }
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

const DSC_DOCSPELL_URL: &str = "DSC_DOCSPELL_URL";
