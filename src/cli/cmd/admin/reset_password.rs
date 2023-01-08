use clap::{Parser, ValueHint};
use snafu::{ResultExt, Snafu};

use super::AdminCmd;
use super::Context;
use crate::cli::sink::Error as SinkError;
use crate::http::payload::{Account, ResetPasswordResp};
use crate::http::Error as HttpError;

/// Resets the password of the given account.
#[derive(Parser, std::fmt::Debug)]
pub struct Input {
    #[arg(long, short, value_hint = ValueHint::Username)]
    pub account: String,
}

impl AdminCmd for Input {
    type CmdError = Error;

    fn exec(&self, admin_opts: &super::Input, ctx: &Context) -> Result<(), Error> {
        let result = reset_password(self, admin_opts, ctx)?;
        ctx.write_result(result).context(WriteResultSnafu)?;
        Ok(())
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An http error occurred: {}", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },

    #[snafu(display("No admin secret provided"))]
    NoAdminSecret,
}

pub fn reset_password(
    input: &Input,
    admin_opts: &super::Input,
    ctx: &Context,
) -> Result<ResetPasswordResp, Error> {
    let secret = super::get_secret(admin_opts, ctx).ok_or(Error::NoAdminSecret)?;
    let account = Account {
        account: input.account.clone(),
    };
    ctx.client
        .admin_reset_password(secret, &account)
        .context(HttpClientSnafu)
}
