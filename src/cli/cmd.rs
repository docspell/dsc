// pub mod admin;
// pub mod cleanup;
// pub mod download;
// pub mod file_exists;
pub mod generate_completions;
// pub mod geninvite;
// pub mod item;
pub mod login;
pub mod logout;
// pub mod register;
pub mod search;
// pub mod search_summary;
// pub mod source;
// pub mod upload;
pub mod version;
// pub mod view;
// pub mod watch;

use super::opts::Format;
use super::sink::{Error as SinkError, Sink};
use crate::cli::opts::CommonOpts;
use crate::config::{ConfigError, DsConfig};
use crate::http::Client;
use serde::Serialize;
use snafu::Snafu;

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
    pub fn new<'a>(opts: &'a CommonOpts, cfg: &'a DsConfig) -> Context<'a> {
        Context {
            opts,
            cfg,
            client: Client::new(docspell_url(opts, cfg)),
        }
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

    fn pass_entry(&self, given: &Option<String>) -> Option<String> {
        given.clone().or(self.cfg.pass_entry.clone())
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
    // Watch {
    //     source: watch::Error,
    // },
    // ItemGet {
    //     source: item::get::Error,
    // },
    Version { source: version::Error },
    // Upload {
    //     source: upload::Error,
    // },
    Login { source: login::Error },

    Logout { source: logout::Error },

    // Cleanup {
    //     source: cleanup::Error,
    // },

    // SourceList {
    //     source: source::list::Error,
    // },

    // SearchSummary {
    //     source: search_summary::Error,
    // },
    Search { source: search::Error },

    // View {
    //     source: view::Error,
    // },

    // Download {
    //     source: download::Error,
    // },

    // Register {
    //     source: register::Error,
    // },

    // GenInvite {
    //     source: geninvite::Error,
    // },

    // FileExists {
    //     source: file_exists::Error,
    // },

    // AdminGeneratePreview {
    //     source: admin::generate_previews::Error,
    // },

    // AdminRecreateIndex {
    //     source: admin::recreate_index::Error,
    // },

    // AdminResetPassword {
    //     source: admin::reset_password::Error,
    // },
    WriteSink { source: SinkError },

    WriteConfig { source: ConfigError },
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
