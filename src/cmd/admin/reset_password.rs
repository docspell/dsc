use crate::cmd::admin::AdminCmd;
use crate::cmd::{CmdArgs, CmdError};
use crate::types::DOCSPELL_ADMIN;
use crate::types::{Account, ResetPasswordResp};
use clap::Clap;

/// Submits a task to re-create the entire fulltext search index.
#[derive(Clap, std::fmt::Debug)]
pub struct Input {
    pub account: String,
}

impl AdminCmd for Input {
    fn exec(&self, secret: &str, args: &CmdArgs) -> Result<(), CmdError> {
        let result = reset_password(secret, self, args)?;
        args.write_result(result)?;
        Ok(())
    }
}

fn reset_password(
    secret: &str,
    input: &Input,
    args: &CmdArgs,
) -> Result<ResetPasswordResp, CmdError> {
    let url = format!("{}/api/v1/admin/user/resetPassword", args.docspell_url());
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
        .json::<ResetPasswordResp>()
        .map_err(CmdError::HttpError)
}
