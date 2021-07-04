use crate::cmd::login;
use crate::cmd::search;
use crate::cmd::search_summary;
use crate::cmd::version;
use clap::{AppSettings, Clap};
use serde::{Deserialize, Serialize};

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
    #[clap(short, long)]
    pub config: Option<String>,

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
}

#[derive(Clap, std::fmt::Debug)]
pub enum SubCommand {
    #[clap(setting = AppSettings::ColoredHelp)]
    Version(version::Input),

    #[clap(setting = AppSettings::ColoredHelp)]
    Login(login::Input),

    #[clap(setting = AppSettings::ColoredHelp)]
    Search(search::Input),

    #[clap(setting = AppSettings::ColoredHelp)]
    SearchSummary(search_summary::Input),
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
