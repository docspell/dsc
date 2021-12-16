use clap::Parser;
use snafu::{ResultExt, Snafu};

use super::{Cmd, Context};
use crate::cli::sink::Error as SinkError;
use crate::http::payload::GenInvite;
use crate::http::Error as HttpError;

/// Generates a new invitation key.
///
/// The password can be found in the config file of the Docspell
/// server.
#[derive(Parser, Debug)]
pub struct Input {
    #[clap(long, short)]
    password: String,
}

impl Cmd for Input {
    type CmdError = Error;

    fn exec(&self, ctx: &Context) -> Result<(), Error> {
        let req = GenInvite {
            password: self.password.clone(),
        };
        let result = ctx.client.gen_invite(&req).context(HttpClient)?;
        ctx.write_result(result).context(WriteResult)?;
        Ok(())
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An http error occurred: {}", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },
}
