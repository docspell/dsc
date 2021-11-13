use clap::Parser;
use snafu::{ResultExt, Snafu};

use super::AdminCmd;
use super::Context;
use crate::cli::sink::Error as SinkError;
use crate::http::Error as HttpError;

/// Submits a task to convert all pdfs via the configured tool (by
/// default ocrmypdf).
#[derive(Parser, std::fmt::Debug)]
pub struct Input {}

impl AdminCmd for Input {
    type CmdError = Error;

    fn exec(&self, admin_opts: &super::Input, ctx: &Context) -> Result<(), Error> {
        let secret = super::get_secret(admin_opts, ctx).ok_or(Error::NoAdminSecret)?;
        let result = ctx
            .client
            .admin_convert_all_pdfs(secret)
            .context(HttpClient)?;
        ctx.write_result(result).context(WriteResult)?;
        Ok(())
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An http error occurred: {}!", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },

    #[snafu(display("No admin secret provided!"))]
    NoAdminSecret,
}
