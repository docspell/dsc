use crate::cmd::login;
use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::types::{SearchResult, DOCSPELL_AUTH};
use clap::Clap;
use snafu::{ResultExt, Snafu};

/// Searches for documents and prints the results.
///
/// Documents are searched via a query. The query syntax is described
/// here: https://docspell.org/docs/query/
#[derive(Clap, std::fmt::Debug)]
pub struct Input {
    /// The query string. See https://docspell.org/docs/query/
    pub query: String,

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
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        let result = search(&self, args).map_err(|source| CmdError::Search { source })?;
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

    #[snafu(display(
        "Error logging in via session. Consider the `login` command. {}",
        source
    ))]
    Login { source: login::Error },
}

pub fn search(opts: &Input, args: &CmdArgs) -> Result<SearchResult, Error> {
    let url = &format!("{}/api/v1/sec/item/search", args.docspell_url());
    let token = login::session_token(args).context(Login)?;
    args.client
        .get(url)
        .header(DOCSPELL_AUTH, token)
        .query(&[
            ("limit", &opts.limit.to_string()),
            ("offset", &opts.offset.to_string()),
            ("withDetails", &opts.with_details.to_string()),
            ("q", &opts.query),
        ])
        .send()
        .and_then(|r| r.error_for_status())
        .context(Http { url })?
        .json::<SearchResult>()
        .context(ReadResponse)
}
