use clap::Parser;
use snafu::{ResultExt, Snafu};

use super::{Cmd, Context};
use crate::cli::sink::Error as SinkError;
use crate::http::Error as HttpError;

/// Performs a search and prints a summary of the results.
#[derive(Parser, std::fmt::Debug)]
pub struct Input {
    /// The query string. See <https://docspell.org/docs/query/>
    query: String,
}

impl Cmd for Input {
    type CmdError = Error;

    fn exec(&self, ctx: &Context) -> Result<(), Error> {
        let result = ctx
            .client
            .summary(&ctx.opts.session, &self.query)
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
}
