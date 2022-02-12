use clap::Parser;
use snafu::{ResultExt, Snafu};

use super::AdminCmd;
use super::Context;
use crate::cli::sink::Error as SinkError;
use crate::http::payload::BasicResult;
use crate::http::Error as HttpError;

/// Submits a task to re-create the entire fulltext search index.
#[derive(Parser, std::fmt::Debug)]
pub struct Input {}

impl AdminCmd for Input {
    type CmdError = Error;

    fn exec(&self, admin_opts: &super::Input, ctx: &Context) -> Result<(), Error> {
        let result = recreate_index(admin_opts, ctx)?;
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

pub fn recreate_index(admin_opts: &super::Input, ctx: &Context) -> Result<BasicResult, Error> {
    let secret = super::get_secret(admin_opts, ctx).ok_or(Error::NoAdminSecret)?;
    ctx.client
        .admin_recreate_index(secret)
        .context(HttpClientSnafu)
}
