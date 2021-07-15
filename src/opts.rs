use crate::cmd::admin;
use crate::cmd::download;
use crate::cmd::file_exists;
use crate::cmd::geninvite;
use crate::cmd::login;
use crate::cmd::register;
use crate::cmd::search;
use crate::cmd::search_summary;
use crate::cmd::source;
use crate::cmd::upload;
use crate::cmd::version;
use crate::cmd::view;
use clap::{AppSettings, ArgGroup, Clap, ValueHint};
use reqwest::blocking::RequestBuilder;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// This is a command line interface to the docspell server. Docspell
/// is a free document management system, designed for home use.
///
/// This CLI is mostly a wrapper around the docspell remote api. For
/// more information, see https://docspell.org/docs/api.
#[derive(Clap, Debug)]
#[clap(name = "dsc", version)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct MainOpts {
    /// This can specify a path to a config file to load. It is
    /// expected to be in TOML format. If not given, the default
    /// config file is looked up based on the current OS. If no such
    /// file exists, the default configuration is used.
    #[clap(short, long, parse(from_os_str), value_hint = ValueHint::FilePath)]
    pub config: Option<PathBuf>,

    #[clap(flatten)]
    pub common_opts: CommonOpts,

    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Clap, Debug)]
pub struct CommonOpts {
    /// Be more verbose when logging.
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: i32,

    /// The output format. Options: json, lisp, csv, tabular. This
    /// defines how to format the output. The default is "Tabular" or
    /// it can be given via the config file. While json and lisp are
    /// always presenting all information, csv and tabular can omit or
    /// consolidate some for better readability.
    #[clap(short, long)]
    pub format: Option<Format>,

    /// The (base) URL to the Docspell server. If not given, it must
    /// be present in the config file.
    #[clap(short, long)]
    pub docspell_url: Option<String>,

    /// Overrides the session token which is by default created by the
    /// `login` command and stored in the file system. It can be
    /// overriden by either the env variable `DSC_SESSION` or using
    /// this option. In these cases, no file system access happens.
    #[clap(long)]
    pub session: Option<String>,
}

#[derive(Clap, std::fmt::Debug)]
pub enum SubCommand {
    /// Write the default config to the file system and exit. The
    /// location depends on the OS.
    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version)]
    WriteDefaultConfig,

    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version)]
    Version(version::Input),

    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version)]
    Login(login::Input),

    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version)]
    Search(search::Input),

    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version)]
    SearchSummary(search_summary::Input),

    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version)]
    FileExists(file_exists::Input),

    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version)]
    GenInvite(geninvite::Input),

    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version)]
    Register(register::Input),

    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version)]
    Source(source::Input),

    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version)]
    Upload(upload::Input),

    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version)]
    Download(download::Input),

    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version)]
    View(view::Input),

    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version)]
    Admin(admin::Input),
}

#[derive(Clap, std::fmt::Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Format {
    Json,
    Lisp,
    Csv,
    Tabular,
}

impl std::str::FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Format, String> {
        if s.eq_ignore_ascii_case("json") {
            Ok(Format::Json)
        } else if s.eq_ignore_ascii_case("lisp") {
            Ok(Format::Lisp)
        } else if s.eq_ignore_ascii_case("tabular") {
            Ok(Format::Tabular)
        } else if s.eq_ignore_ascii_case("csv") {
            Ok(Format::Csv)
        } else {
            Err(format!("Invalid format string: {}", s))
        }
    }
}

#[derive(Clap, Debug)]
#[clap(group = ArgGroup::new("int"))]
pub struct EndpointOpts {
    /// When using the integration endpoint, provides the Basic auth
    /// header as credential. This must be a `username:password` pair.
    #[clap(long, group = "int")]
    pub basic: Option<NameVal>,

    /// Use the integration endpoint and provide the http header as
    /// credentials. This must be a `Header:Value` pair.
    #[clap(long, group = "int")]
    pub header: Option<NameVal>,

    /// Use the integration endpoint. Credentials `--header|--basic`
    /// must be specified if applicable.
    #[clap(long, short)]
    pub integration: bool,

    /// When using the integration endpoint, the collective is required.
    #[clap(long, short)]
    pub collective: Option<String>,
}

impl EndpointOpts {
    pub fn apply(&self, c: RequestBuilder) -> RequestBuilder {
        self.apply_basic(self.apply_header(c))
    }

    fn apply_basic(&self, c: RequestBuilder) -> RequestBuilder {
        if let Some(b) = &self.basic {
            log::debug!(
                "Using integration endpoint with basic auth: {}:{}",
                b.name,
                b.value
            );
            c.basic_auth(&b.name, Some(&b.value))
        } else {
            c
        }
    }

    fn apply_header(&self, c: RequestBuilder) -> RequestBuilder {
        if let Some(h) = &self.header {
            log::debug!(
                "Using integration endpoint with header: {}:{}",
                h.name,
                h.value
            );
            c.header(&h.name, &h.value)
        } else {
            c
        }
    }
}

#[derive(Debug)]
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

#[derive(Clap, Debug)]
pub struct UploadMeta {
    /// If set, all files are uploaded as one single item. Default is
    /// to create one item per file.
    #[clap(long = "single-item", parse(from_flag = std::ops::Not::not))]
    pub multiple: bool,

    /// Specify the direction of the item. One of: outgoing, incoming.
    #[clap(long)]
    pub direction: Option<String>,

    /// Specify a folder to associate to the new item.
    #[clap(long)]
    pub folder: Option<String>,

    /// Alow duplicates by skipping the duplicate check.
    #[clap(long = "allow-dupes", parse(from_flag = std::ops::Not::not))]
    pub skip_duplicates: bool,

    /// Specify a list of tags to associate. Tags can be given by name
    /// or id. The option can be repeated multiple times.
    #[clap(long, required = false, min_values = 1, number_of_values = 1)]
    pub tag: Vec<String>,

    /// Only applicable for zip/eml files. Specify a file filter to
    /// use when unpacking archives like zip files or eml files.
    #[clap(long)]
    pub file_filter: Option<String>,

    /// Specify the language of the document.
    #[clap(long, short)]
    pub language: Option<String>,
}
