use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::config::DsConfig;
use clap::Clap;
use serde::{Deserialize, Serialize};

/// Queries the server for its version information.
#[derive(Clap, std::fmt::Debug)]
pub struct Input {}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        let result = version(args.cfg).and_then(|r| args.make_str(&r));
        println!("{:}", result?);
        Ok(())
    }
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

fn version(cfg: &DsConfig) -> Result<VersionDto, CmdError> {
    let url = format!("{}/api/info/version", cfg.docspell_url);
    return reqwest::blocking::get(url)
        .map_err(CmdError::HttpError)?
        .json::<VersionDto>()
        .map_err(CmdError::HttpError);
}
