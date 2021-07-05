use crate::cmd::admin::AdminCmd;
use crate::cmd::{CmdArgs, CmdError};
use crate::opts::ConfigOpts;
use crate::types::DOCSPELL_ADMIN;
use clap::Clap;
use serde::{Deserialize, Serialize};

/// Submits a task to re-create the entire fulltext search index.
#[derive(Clap, std::fmt::Debug)]
pub struct Input {
    pub account: String,
}

impl AdminCmd for Input {
    fn exec(&self, secret: &str, args: &CmdArgs) -> Result<(), CmdError> {
        let result = reset_password(secret, self, args.opts).and_then(|r| args.make_str(&r));
        println!("{:}", result?);
        Ok(())
    }
}

fn reset_password(secret: &str, input: &Input, cfg: &ConfigOpts) -> Result<Response, CmdError> {
    let url = format!("{}/api/v1/admin/user/resetPassword", cfg.docspell_url);
    let account = Account {
        account: input.account.clone(),
    };
    let client = reqwest::blocking::Client::new();
    client
        .post(url)
        .header(DOCSPELL_ADMIN, secret)
        .json(&account)
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(CmdError::HttpError)?
        .json::<Response>()
        .map_err(CmdError::HttpError)
}

#[derive(Debug, Serialize, Deserialize)]
struct Account {
    account: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub success: bool,
    pub message: String,
    #[serde(alias = "newPassword")]
    pub new_password: String,
}
