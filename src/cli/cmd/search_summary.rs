use crate::cmd::login;
use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::types::{Summary, DOCSPELL_AUTH};
use clap::Clap;
use snafu::{ResultExt, Snafu};

/// Performs a search and prints a summary of the results.
#[derive(Clap, std::fmt::Debug)]
pub struct Input {
    /// The query string. See https://docspell.org/docs/query/
    query: String,
}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        let result = summary(&self, args).map_err(|source| CmdError::SearchSummary { source })?;
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

pub fn summary(opts: &Input, args: &CmdArgs) -> Result<Summary, Error> {
    let url = &format!("{}/api/v1/sec/item/searchStats", args.docspell_url());
    let token = login::session_token(args).context(Login)?;
    args.client
        .get(url)
        .header(DOCSPELL_AUTH, token)
        .query(&[("q", &opts.query)])
        .send()
        .and_then(|r| r.error_for_status())
        .context(Http { url })?
        .json::<Summary>()
        .context(ReadResponse)
}
