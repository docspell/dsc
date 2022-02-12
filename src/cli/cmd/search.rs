use clap::Parser;
use snafu::{ResultExt, Snafu};

use super::{Cmd, Context};
use crate::cli::opts::SearchMode;
use crate::cli::sink::Error as SinkError;
use crate::http::payload::{SearchReq, SearchResult};
use crate::http::Error as HttpError;

/// Searches for documents and prints the results.
///
/// Documents are searched via a query. The query syntax is described
/// here: <https://docspell.org/docs/query/>
#[derive(Parser, std::fmt::Debug)]
pub struct Input {
    /// The query string. See <https://docspell.org/docs/query/>
    pub query: String,

    #[clap(flatten)]
    pub search_mode: SearchMode,

    /// Do not fetch details to each item in the result
    #[clap(long = "no-details", parse(from_flag = std::ops::Not::not))]
    pub with_details: bool,

    /// Limit the number of results.
    #[clap(short, long, default_value = "20")]
    pub limit: u32,

    /// Skip the first n results.
    #[clap(short, long, default_value = "0")]
    pub offset: u32,
}

impl Cmd for Input {
    type CmdError = Error;

    fn exec(&self, ctx: &Context) -> Result<(), Error> {
        let result = search(self, ctx)?;
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

fn search(opts: &Input, ctx: &Context) -> Result<SearchResult, Error> {
    let req = SearchReq {
        limit: opts.limit,
        offset: opts.offset,
        with_details: opts.with_details,
        query: opts.query.clone(),
        search_mode: opts.search_mode.to_mode(),
    };

    ctx.client
        .search(&ctx.opts.session, &req)
        .context(HttpClientSnafu)
}
