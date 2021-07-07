use crate::cmd::login;
use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::opts::ConfigOpts;
use crate::types::{SourceList, DOCSPELL_AUTH};
use clap::Clap;

/// List all sources for your collective
#[derive(Clap, std::fmt::Debug)]
pub struct Input {}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        let result = list_sources(args.opts).and_then(|r| args.make_str(&r));
        println!("{:}", result?);
        Ok(())
    }
}

fn list_sources(cfg: &ConfigOpts) -> Result<SourceList, CmdError> {
    let url = format!("{}/api/v1/sec/source", cfg.docspell_url);
    let client = reqwest::blocking::Client::new();
    let token = login::session_token(cfg)?;
    client
        .get(url)
        .header(DOCSPELL_AUTH, token)
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(CmdError::HttpError)?
        .json::<SourceList>()
        .map_err(CmdError::HttpError)
}
