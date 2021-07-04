use crate::cmd::login;
use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::config::DsConfig;
use crate::types::SearchResult;
use clap::Clap;

#[derive(Clap, std::fmt::Debug)]
pub struct Input {
    query: String,
    #[clap(long)]
    with_details: bool,
    #[clap(short, long, default_value = "20")]
    limit: u32,
    #[clap(short, long, default_value = "0")]
    offset: u32,
}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        let result = search(&self, args.cfg).and_then(|r| args.make_str(&r));
        println!("{:}", result?);
        Ok(())
    }
}

fn search(args: &Input, cfg: &DsConfig) -> Result<SearchResult, CmdError> {
    let url = format!("{}/api/v1/sec/item/search", cfg.docspell_url);
    let client = reqwest::blocking::Client::new();
    let token = login::session_token()?;
    client
        .get(url)
        .header(login::DOCSPELL_AUTH, token)
        .query(&[
            ("limit", &args.limit.to_string()),
            ("offset", &args.offset.to_string()),
            ("withDetails", &args.with_details.to_string()),
            ("q", &args.query),
        ])
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(CmdError::HttpError)?
        .json::<SearchResult>()
        .map_err(CmdError::HttpError)
}
