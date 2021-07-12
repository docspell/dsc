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
    sink::Sink,
};
use serde::Serialize;
use snafu::{ResultExt, Snafu};

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
        Sink::write_value(fmt, &value).context(SinkError)?;
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

    fn pass_entry(&self, given: &Option<String>) -> Option<String> {
        given.clone().or(self.cfg.pass_entry.clone())
    }
}

#[derive(Debug, Snafu)]
pub enum CmdError {
    Version {
        source: version::Error,
    },

    Upload {
        source: upload::Error,
    },

    Login {
        source: login::Error,
    },

    SourceList {
        source: source::list::Error,
    },

    SearchSummary {
        source: search_summary::Error,
    },

    Search {
        source: search::Error,
    },

    Register {
        source: register::Error,
    },

    GenInvite {
        source: geninvite::Error,
    },

    FileExists {
        source: file_exists::Error,
    },

    AdminGeneratePreview {
        source: admin::generate_previews::Error,
    },

    AdminRecreateIndex {
        source: admin::recreate_index::Error,
    },

    AdminResetPassword {
        source: admin::reset_password::Error,
    },

    SinkError {
        source: crate::sink::Error,
    },
}
