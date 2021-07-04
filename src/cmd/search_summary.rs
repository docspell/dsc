use crate::cmd::login;
use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::config::DsConfig;
use crate::types::Summary;
use clap::Clap;

#[derive(Clap, std::fmt::Debug)]
pub struct Input {
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
    let token = login::session_token()?;
    client
        .get(url)
        .header(login::DOCSPELL_AUTH, token)
        .query(&[("q", &args.query)])
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(CmdError::HttpError)?
        .json::<Summary>()
        .map_err(CmdError::HttpError)
}
