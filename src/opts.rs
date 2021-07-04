use crate::cmd::login;
use crate::cmd::search;
use crate::cmd::version;
use clap::{AppSettings, Clap};
use serde::{Deserialize, Serialize};

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Clap, std::fmt::Debug)]
#[clap(version = VERSION)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct MainOpts {
    #[clap(short, long, about("Specify a config file to load"))]
    pub config: Option<String>,

    #[clap(flatten)]
    pub common_opts: CommonOpts,

    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Clap, Debug)]
pub struct CommonOpts {
    #[clap(
        short,
        long,
        parse(from_occurrences),
        about("Be more verbose when logging")
    )]
    pub verbose: i32,

    #[clap(short, long, about("Specify an output format. One of: json, lisp"))]
    pub format: Option<Format>,
}

#[derive(Clap, std::fmt::Debug)]
pub enum SubCommand {
    #[clap(about("Prints the server version"))]
    Version(version::Input),

    #[clap(about("Login into the Docspell server"))]
    Login(login::Input),

    #[clap(about("Search for documents"))]
    Search(search::Input),
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
