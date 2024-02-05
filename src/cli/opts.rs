//! Defines all options and commands for the cli via [clap](https://clap.rs).

use super::cmd::*;
use crate::{
    config::DsConfig,
    http::payload,
    http::proxy,
    http::{FileAuth, IntegrationAuth, IntegrationData},
};
use clap::{ArgAction, ArgGroup, Parser, ValueEnum, ValueHint};
use serde::{Deserialize, Serialize};
use snafu::Snafu;
use std::{path::PathBuf, str::FromStr};

/// This is a command line interface to the docspell server. Docspell
/// is a free document management system, designed for home use.
///
/// This CLI is mostly a wrapper around the docspell remote api. For
/// more information, see <https://docspell.org/docs/api>.
#[derive(Parser, Debug)]
#[command(name = "dsc", version)]
pub struct MainOpts {
    /// This can specify a path to a config file to load. It is
    /// expected to be in TOML format. If not given, the default
    /// config file is looked up based on the current OS. If no such
    /// file exists, the default configuration is used.
    ///
    /// The environment variable DSC_CONFIG can also be used to define
    /// a specific config file.
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    pub config: Option<PathBuf>,

    #[clap(flatten)]
    pub common_opts: CommonOpts,

    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

/// Options that are applicable to all (or most) commands.
///
/// They are defined for the main command, before the subcommand is
/// defined.
#[derive(Parser, Debug)]
#[command(group = ArgGroup::new("tls"))]
pub struct CommonOpts {
    /// Be more verbose when logging. Verbosity is increased by
    /// occurrence of this option. Use `-vv` for debug verbosity and
    /// `-v` for info.
    #[arg(short, long, action = ArgAction::Count)]
    pub verbose: u8,

    /// The output format. This defines how to format the output. The
    /// default is "Tabular" or it can be given via the config file.
    /// While json and lisp are always presenting all information, csv
    /// and tabular can omit or consolidate some for better
    /// readability.
    #[arg(short, long, value_enum)]
    pub format: Option<Format>,

    /// The (base) URL to the Docspell server. If not given, it must
    /// be present in the config file or given as environment variable
    /// DSC_DOCSPELL_URL.
    #[arg(short, long, value_hint = ValueHint::Url)]
    pub docspell_url: Option<String>,

    /// Overrides the session token which is by default created by the
    /// `login` command and stored in the file system. It can be
    /// overriden by either the env variable `DSC_SESSION` or using
    /// this option. In these cases, no file system access happens.
    #[arg(long)]
    pub session: Option<String>,

    /// Set a proxy to use for doing http requests. By default, the
    /// system proxy will be used. Can be either `none` or <url>. If
    /// `none`, the system proxy will be ignored; otherwise specify
    /// the proxy url, like `http://myproxy.com`.
    #[arg(long)]
    pub proxy: Option<ProxySetting>,

    /// The user to authenticate at the proxy via Basic auth.
    #[arg(long)]
    pub proxy_user: Option<String>,

    /// The password to authenticate at the proxy via Basic auth.
    #[arg(long)]
    pub proxy_password: Option<String>,

    /// Add a root certificate to the trust store used when connecting
    /// via TLS. It can be a PEM or DER formatted file.
    #[arg(long, value_hint = ValueHint::FilePath, group = "tls")]
    pub extra_certificate: Option<PathBuf>,

    /// This ignores any invalid certificates when connecting to the
    /// docspell server. It is obvious, that this should be used
    /// carefully! This cannot be used, when `--extra-certificate` is
    /// specfified.
    #[arg(long, group = "tls")]
    pub accept_invalid_certificates: bool,
}

impl CommonOpts {
    pub fn to_proxy_setting(
        proxy: &Option<ProxySetting>,
        user: Option<String>,
        password: Option<String>,
    ) -> proxy::ProxySetting {
        match proxy {
            None => proxy::ProxySetting::System,
            Some(ProxySetting::None) => proxy::ProxySetting::None,
            Some(ProxySetting::Custom { url }) => proxy::ProxySetting::Custom {
                url: url.clone(),
                user,
                password,
            },
        }
    }
}

#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
pub enum ProxySetting {
    /// Don't use any proxy; this will also discard the system proxy.
    None,

    /// Use a custom defined proxy.
    Custom { url: String },
}

impl FromStr for ProxySetting {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("none") {
            Ok(ProxySetting::None)
        } else {
            Ok(ProxySetting::Custom { url: s.to_string() })
        }
    }
}

/// All subcommands.
#[derive(Parser, Debug)]
pub enum SubCommand {
    /// Write the default config to the file system and exit.
    ///
    /// The location depends on the OS and is shown after writing.
    #[command(version)]
    WriteDefaultConfig,

    /// Write completions for some shells to stdout.
    GenerateCompletions(generate_completions::Input),

    #[command(version)]
    Watch(watch::Input),

    #[command(version)]
    Version(version::Input),

    #[command(version)]
    Login(login::Input),

    #[command(version)]
    Logout(logout::Input),

    #[command(version)]
    Search(search::Input),

    #[command(version, alias = "summary")]
    SearchSummary(search_summary::Input),

    #[command(version)]
    FileExists(file_exists::Input),

    #[command(version)]
    GenInvite(geninvite::Input),

    #[command(version)]
    Register(register::Input),

    #[command(version)]
    Source(source::Input),

    #[command(version)]
    Item(item::Input),

    #[command(version)]
    Bookmark(bookmark::Input),

    #[command(version, alias = "up")]
    Upload(upload::Input),

    #[command(version)]
    Download(download::Input),

    #[command(version)]
    View(view::Input),

    #[command(version)]
    Cleanup(cleanup::Input),

    #[command(version)]
    Export(export::Input),

    #[command(version)]
    Admin(admin::Input),

    #[command(version)]
    OpenItem(open_item::Input),
}

/// The format for presenting the results.
#[derive(ValueEnum, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Format {
    Json,
    Lisp,
    Elisp,
    Csv,
    Tabular,
}

// Shared options regarding file uploads or file existence checks.
//
// Uploads can be done via a source id, the integration endpoint or a
// valid session.
#[derive(Parser, Debug, Clone)]
#[command(group = ArgGroup::new("int"))]
#[command(group = ArgGroup::new("g_source"))]
pub struct EndpointOpts {
    /// Use the integration endpoint and provide the basic auth header
    /// as credentials. This must be a `username:password` pair as the
    /// first line not starting with '#'.
    #[arg(long, group = "int", value_hint = ValueHint::FilePath)]
    pub basic_file: Option<PathBuf>,

    /// Use the integration endpoint and provide the http header as
    /// credentials. This must be a `Header:Value` pair as the first
    /// line not starting with '#'.
    #[arg(long, group = "int", value_hint = ValueHint::FilePath)]
    pub header_file: Option<PathBuf>,

    /// When using the integration endpoint, provides the Basic auth
    /// header as credential. This must be a `username:password` pair.
    #[arg(long, group = "int")]
    pub basic: Option<NameVal>,

    /// Use the integration endpoint and provide the http header as
    /// credentials. This must be a `Header:Value` pair.
    #[arg(long, group = "int")]
    pub header: Option<NameVal>,

    /// Use the integration endpoint. Credentials
    /// `--header[-file]|--basic[-file]` must be specified if
    /// applicable.
    #[arg(long, short, group = "g_source")]
    pub integration: bool,

    /// When using the integration endpoint, the collective is
    /// required.
    #[arg(long, short, value_hint = ValueHint::Username)]
    pub collective: Option<String>,

    /// Use the given source id. If not specified, the default id from
    /// the config is used; otherwise a login is required
    #[arg(long, group = "int")]
    #[arg(group = "g_source")]
    pub source: Option<String>,
}

#[derive(Debug, Snafu)]
pub enum FileAuthError {
    #[snafu(display("Could not read file: {}", path.display()))]
    FileRead {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Could not parse name:value pair in '{}': {}", path.display(), message))]
    NameValParse { path: PathBuf, message: String },

    #[snafu(display("No collective specified"))]
    NoCollective,
}

impl EndpointOpts {
    pub fn get_source_id(&self, cfg: &DsConfig) -> Option<String> {
        self.source
            .clone()
            .or_else(|| cfg.default_source_id.clone())
    }

    fn read_name_val(file: &PathBuf) -> Result<NameVal, FileAuthError> {
        let cnt = std::fs::read_to_string(file).map_err(|e| FileAuthError::FileRead {
            path: file.to_path_buf(),
            source: e,
        })?;

        let line = cnt
            .lines()
            .filter(|s| !s.starts_with("#"))
            .take(1)
            .map(String::from)
            .nth(0);

        match line {
            Some(l) => NameVal::from_str(&l).map_err(|str| FileAuthError::NameValParse {
                path: file.to_path_buf(),
                message: str,
            }),
            None => Err(FileAuthError::NameValParse {
                path: file.to_path_buf(),
                message: "File is empty".to_string(),
            }),
        }
    }

    /// Convert the options into a `FileAuth` object to be used with the http client
    pub fn to_file_auth(
        &self,
        ctx: &Context,
        fallback_cid: &dyn Fn() -> Option<String>,
    ) -> Result<FileAuth, FileAuthError> {
        if self.integration {
            let cid = self
                .collective
                .clone()
                .or_else(fallback_cid)
                .ok_or(FileAuthError::NoCollective)?;
            let mut res = IntegrationData {
                collective: cid,
                auth: IntegrationAuth::None,
            };
            if let Some(header_file) = &self.header_file {
                log::debug!(
                    "Reading file for integration header {}",
                    header_file.display()
                );
                let np = Self::read_name_val(&header_file)?;
                res.auth = IntegrationAuth::Header(np.name.clone(), np.value.clone());
            }
            if let Some(basic_file) = &self.basic_file {
                log::debug!("Reading file for basic auth {}", basic_file.display());
                let np = Self::read_name_val(&basic_file)?;
                res.auth = IntegrationAuth::Basic(np.name.clone(), np.value.clone());
            }
            if let Some(basic) = &self.basic {
                res.auth = IntegrationAuth::Basic(basic.name.clone(), basic.value.clone());
            }
            if let Some(header) = &self.header {
                res.auth = IntegrationAuth::Header(header.name.clone(), header.value.clone());
            }
            Ok(FileAuth::Integration(res))
        } else {
            let sid = self.get_source_id(ctx.cfg);
            match sid {
                Some(id) => Ok(FileAuth::from_source(id)),
                None => Ok(FileAuth::Session {
                    token: ctx.opts.session.clone(),
                }),
            }
        }
    }
}

/// A generic name + value structure.
///
/// Used for parsing two arguments from a single string. Values must
/// be separated by a colon `:`.
#[derive(Debug, Clone)]
pub struct NameVal {
    pub name: String,
    pub value: String,
}

impl std::str::FromStr for NameVal {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pos = s
            .find(':')
            .ok_or_else(|| format!("Not a key:value pair, no `:` found in '{}'", s))?;
        Ok(NameVal {
            name: s[..pos].into(),
            value: s[pos + 1..].into(),
        })
    }
}

/// The direction of an item in docspell.
#[derive(ValueEnum, Debug, Clone)]
pub enum Direction {
    In,
    Out,
}
impl Direction {
    pub fn to_value(&self) -> &'static str {
        match self {
            Direction::In => "incoming",
            Direction::Out => "outgoung",
        }
    }
}

// Shared options used for uploading.
#[derive(Parser, Debug, Clone)]
pub struct UploadMeta {
    /// Specify the direction of the item.
    #[arg(long, value_enum)]
    pub direction: Option<Direction>,

    /// Specify a folder to associate to the new item.
    #[arg(long)]
    pub folder: Option<String>,

    /// Alow duplicates by skipping the duplicate check.
    #[arg(long = "allow-dupes", action = ArgAction::SetFalse)]
    pub skip_duplicates: bool,

    /// Specify a list of tags to associate. Tags can be given by name
    /// or id. The option can be repeated multiple times.
    #[arg(long, required = false, num_args = 1)]
    pub tag: Vec<String>,

    /// Only applicable for zip/eml files. Specify a file filter to
    /// use when unpacking archives like zip files or eml files.
    #[arg(long)]
    pub file_filter: Option<String>,

    /// Specify the language of the document.
    #[arg(long, short)]
    pub language: Option<String>,

    /// Discard the the mail body and only import the attachments.
    /// Only applicable when e-mail files are uploaded.
    #[arg(long)]
    pub attachments_only: bool,

    /// If specified, extracts zip files and submits a separate job
    /// for each entry. Otherwise zip files are treated as related and
    /// result in a single item each with perhaps multiple
    /// attachments.
    #[arg(long)]
    pub flatten_archives: bool,
}

// Shared options for specifying what to do with a file.
#[derive(Parser, Debug, Clone)]
#[command(group = ArgGroup::new("file-action"))]
pub struct FileAction {
    /// Deletes the file.
    #[arg(long, group = "file-action")]
    pub delete: bool,

    /// Moves the file into the given directory. The directory
    /// structure is retained in the target folder.
    #[arg(long = "move", group = "file-action", value_hint = ValueHint::DirPath)]
    pub move_to: Option<PathBuf>,
}

#[derive(Parser, Debug, Clone)]
#[command(group = ArgGroup::new("search-mode"))]
pub struct SearchMode {
    /// Search in trashed items, too.
    #[arg(long, group = "search-mode")]
    pub all: bool,

    /// Search only in trashed items.
    #[arg(long, group = "search-mode")]
    pub trashed_only: bool,
}

impl SearchMode {
    pub fn to_mode(&self) -> payload::SearchMode {
        if self.all {
            payload::SearchMode::All
        } else if self.trashed_only {
            payload::SearchMode::Trashed
        } else {
            payload::SearchMode::Normal
        }
    }
}
