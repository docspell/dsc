use crate::cmd::{admin, CmdArgs, CmdError};
use crate::types::DOCSPELL_ADMIN;
use crate::types::{Account, ResetPasswordResp};
use clap::Clap;
use snafu::{ResultExt, Snafu};

use super::AdminCmd;

/// Resets the password of the given account.
#[derive(Clap, std::fmt::Debug)]
pub struct Input {
    pub account: String,
}

impl AdminCmd for Input {
    fn exec(&self, admin_opts: &admin::Input, args: &CmdArgs) -> Result<(), CmdError> {
        let result = reset_password(self, admin_opts, args)
            .map_err(|source| CmdError::AdminResetPassword { source })?;
        args.write_result(result)?;
        Ok(())
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error received from server at {}: {}", url, source))]
    Http { source: reqwest::Error, url: String },

    #[snafu(display("Error received from server: {}", source))]
    ReadResponse { source: reqwest::Error },

    #[snafu(display("No admin secret provided!"))]
    NoAdminSecret,
}

pub fn reset_password(
    input: &Input,
    admin_opts: &admin::Input,
    args: &CmdArgs,
) -> Result<ResetPasswordResp, Error> {
    let secret = admin::get_secret(admin_opts, args).ok_or(Error::NoAdminSecret)?;
    let url = &format!("{}/api/v1/admin/user/resetPassword", args.docspell_url());
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
        .context(Http { url })?
        .json::<ResetPasswordResp>()
        .context(ReadResponse)
}
