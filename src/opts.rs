use crate::cmd::admin;
use crate::cmd::file_exists;
use crate::cmd::login;
use crate::cmd::search;
use crate::cmd::search_summary;
use crate::cmd::source;
use crate::cmd::version;
use crate::config::DsConfig;
use clap::{AppSettings, Clap, ValueHint};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

/// This is a command line interface to the docspell server. Docspell
/// is a free document management system, designed for home use.
///
/// This program is meant to be used from other programs. As such, the
/// main output format is either JSON or S-EXPR (lisp). Sub commands
/// are usually mapped to a corresponding API endpoint on the server.
///
/// For more information, see https://docspell.org/docs/api.
#[derive(Clap, std::fmt::Debug)]
#[clap(name = "dsc", version = VERSION)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct MainOpts {
    /// This can specify a path to a config file to load. It is
    /// expected to be in TOML format. If not given, the default
    /// config file is looked up based on the current OS. If no such
    /// file exists, the default configuration is used and a new
    /// config file is created from that.
    #[clap(short, long, parse(from_os_str), value_hint = ValueHint::FilePath)]
    pub config: Option<PathBuf>,

    #[clap(flatten)]
    pub common_opts: CommonOpts,

    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Clap, Debug)]
pub struct CommonOpts {
    /// Be more verbose when logging. Enable logging via env variable RUST_LOG=debug|info.
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: i32,

    /// The output format. Some commands may ignore this option. This
    /// defines how to format the output. The default is JSON or give
    /// via the config file. Another option is `Lisp` which produces
    /// s-expressions. Use one of: json, lisp.
    #[clap(short, long)]
    pub format: Option<Format>,

    /// For commands using the admin endpoint, this is the secret
    /// required to access them. If not given here, it is taken from
    /// the config file.
    #[clap(short, long)]
    pub admin_secret: Option<String>,

    /// The (base) URL to the Docspell server. If not given, it must
    /// be present in the config file.
    #[clap(short, long)]
    pub docspell_url: Option<String>,
}

// CommonOpts with fallback from DsConfig
#[derive(Debug)]
pub struct ConfigOpts {
    pub verbose: i32,
    pub format: Format,
    pub docspell_url: String,
    pub admin_secret: Option<String>,
}

impl CommonOpts {
    pub fn merge(&self, cfg: &DsConfig) -> ConfigOpts {
        ConfigOpts {
            verbose: self.verbose,
            format: self.format.unwrap_or(cfg.default_format),
            docspell_url: self
                .docspell_url
                .as_ref()
                .unwrap_or(&cfg.docspell_url)
                .clone(),
            admin_secret: self
                .admin_secret
                .as_ref()
                .or(cfg.admin_secret.as_ref())
                .map(String::clone),
        }
    }
}

#[derive(Clap, std::fmt::Debug)]
pub enum SubCommand {
    /// Write the default config to the file system and exit. The
    /// location depends on the OS.
    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version = VERSION)]
    WriteDefaultConfig,

    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version = VERSION)]
    Version(version::Input),

    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version = VERSION)]
    Login(login::Input),

    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version = VERSION)]
    Search(search::Input),

    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version = VERSION)]
    SearchSummary(search_summary::Input),

    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version = VERSION)]
    FileExists(file_exists::Input),

    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version = VERSION)]
    Source(source::Input),

    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version = VERSION)]
    Admin(admin::Input),
}

#[derive(Clap, std::fmt::Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Format {
    Json,
    Lisp,
}

impl std::str::FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Format, String> {
        if s.eq_ignore_ascii_case("json") {
            Ok(Format::Json)
        } else if s.eq_ignore_ascii_case("lisp") {
            Ok(Format::Lisp)
        } else {
            Err(format!("Invalid format string: {}", s))
        }
    }
}
