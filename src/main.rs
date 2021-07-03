use clap::{AppSettings, Clap};
use dsc::config::DsConfig;
use serde::{Deserialize, Serialize};

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let opts = MainOpts::parse();
    println!(
        "Using config: {:?}, verbosity={}",
        opts.config, opts.verbose
    );

    let cfg: DsConfig = dsc::read_config(&opts.config).expect("Config could not be read");

    println!("base={:?}", cfg.docspell_url);

    match opts.subcmd {
        SubCommand::Version => {
            println!("{:#?}", version());
        }
    }
}

#[derive(Clap)]
#[clap(version = VERSION)]
#[clap(setting = AppSettings::ColoredHelp)]
struct MainOpts {
    #[clap(short, long)]
    config: Option<String>,

    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,

    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    Version,
}

#[derive(Debug, Serialize, Deserialize)]
struct VersionDto {
    version: String,
    #[serde(alias = "builtAtMillis")]
    built_at_millis: i64,
    #[serde(alias = "builtAtString")]
    built_at_string: String,
    #[serde(alias = "gitCommit")]
    git_commit: String,
    #[serde(alias = "gitVersion")]
    git_version: String,
}

fn version() -> Result<VersionDto, reqwest::Error> {
    return reqwest::blocking::get("https://docs.daheim.site/api/info/version")?
        .json::<VersionDto>();
}
