use clap::Parser;
use snafu::{ResultExt, Snafu};

use super::AdminCmd;
use super::Context;
use crate::cli::sink::Error as SinkError;
use crate::http::payload::FileCloneRequest;
use crate::http::Error as HttpError;

/// Submits a task to clone the default file repository to a different
/// one.
#[derive(Parser, std::fmt::Debug)]
pub struct Input {
    #[clap(long, short)]
    pub target: Vec<String>,
}

impl AdminCmd for Input {
    type CmdError = Error;

    fn exec(&self, admin_opts: &super::Input, ctx: &Context) -> Result<(), Error> {
        let secret = super::get_secret(admin_opts, ctx).ok_or(Error::NoAdminSecret)?;
        let req = FileCloneRequest {
            target_repositories: self.target.clone(),
        };
        log::info!("Sending task to clone file repository to: {:?}", req);
        let result = ctx
            .client
            .admin_files_clone_repository(secret, &req)
            .context(HttpClientSnafu)?;
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
