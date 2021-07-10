use crate::cmd::admin::AdminCmd;
use crate::cmd::{CmdArgs, CmdError};
use crate::types::BasicResult;
use crate::types::DOCSPELL_ADMIN;
use clap::Clap;

/// Submits a task to re-create the entire fulltext search index.
#[derive(Clap, std::fmt::Debug)]
pub struct Input {}

impl AdminCmd for Input {
    fn exec(&self, secret: &str, args: &CmdArgs) -> Result<(), CmdError> {
        let result = recreate_index(secret, args).and_then(|r| args.make_str(&r));
        println!("{:}", result?);
        Ok(())
    }
}

fn recreate_index(secret: &str, args: &CmdArgs) -> Result<BasicResult, CmdError> {
    let url = format!("{}/api/v1/admin/fts/reIndexAll", args.docspell_url());
    let client = reqwest::blocking::Client::new();
    client
        .post(url)
        .header(DOCSPELL_ADMIN, secret)
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(CmdError::HttpError)?
        .json::<BasicResult>()
        .map_err(CmdError::HttpError)
}
