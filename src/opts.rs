use crate::cmd::version;
use clap::{AppSettings, Clap};

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Clap, std::fmt::Debug)]
#[clap(version = VERSION)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct MainOpts {
    #[clap(short, long, about("Specify a config file to load"))]
    pub config: Option<String>,

    #[clap(
        short,
        long,
        parse(from_occurrences),
        about("Be more verbose when logging")
    )]
    pub verbose: i32,

    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Clap, std::fmt::Debug)]
pub enum SubCommand {
    #[clap(about("Prints the server version"))]
    Version(version::Input),
}

#[derive(Clap, std::fmt::Debug)]
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
