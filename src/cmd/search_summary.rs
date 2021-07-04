use crate::cmd::login;
use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::config::DsConfig;
use crate::types::{Summary, DOCSPELL_AUTH};
use clap::Clap;

/// Performs a search and prints a summary of the results.
#[derive(Clap, std::fmt::Debug)]
pub struct Input {
    /// The query string. See https://docspell.org/docs/query/
    query: String,
}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        let result = summary(&self, args.cfg).and_then(|r| args.make_str(&r));
        println!("{:}", result?);
        Ok(())
    }
}

fn summary(args: &Input, cfg: &DsConfig) -> Result<Summary, CmdError> {
    let url = format!("{}/api/v1/sec/item/searchStats", cfg.docspell_url);
    let client = reqwest::blocking::Client::new();
    let token = login::session_token(cfg)?;
    client
        .get(url)
        .header(DOCSPELL_AUTH, token)
        .query(&[("q", &args.query)])
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(CmdError::HttpError)?
        .json::<Summary>()
        .map_err(CmdError::HttpError)
}
