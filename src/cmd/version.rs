use crate::cmd::{Cmd, CmdError};
use crate::config::DsConfig;
use crate::opts::Format;
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Clap, std::fmt::Debug)]
pub struct Input {
    #[clap(short, long)]
    pub format: Option<Format>,
}

impl Cmd for Input {
    fn exec(&self, cfg: DsConfig) -> Result<(), CmdError> {
        let resp = version(&cfg);
        let result = match self.format {
            Some(Format::Json) => resp.and_then(|r| Self::json_str(&r)),
            Some(Format::Lisp) => resp.and_then(|r| Self::sexpr_str(&r)),
            None => resp.and_then(|r| Self::json_str(&r)),
        };
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
