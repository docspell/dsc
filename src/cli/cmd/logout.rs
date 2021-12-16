use clap::Parser;
use snafu::{ResultExt, Snafu};

use super::{Cmd, Context};
use crate::cli::sink::Error as SinkError;
use crate::http::payload::BasicResult;
use crate::http::Error as HttpError;

/// Removes the credentials file
#[derive(Parser, Debug)]
pub struct Input {}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An http error occurred: {}", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },
}

impl Cmd for Input {
    type CmdError = Error;

    fn exec(&self, ctx: &Context) -> Result<(), Error> {
        ctx.client.logout().context(HttpClient)?;
        let message = BasicResult {
            success: true,
            message: "Session deleted.".into(),
        };
        ctx.write_result(message).context(WriteResult)?;
        Ok(())
    }
}
