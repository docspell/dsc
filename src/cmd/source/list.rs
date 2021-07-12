use crate::cmd::login;
use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::types::{SourceAndTags, SourceList, DOCSPELL_AUTH};
use clap::Clap;
use snafu::{ResultExt, Snafu};

/// List all sources for your collective
#[derive(Clap, std::fmt::Debug)]
pub struct Input {
    /// Filter sources that start by the given name
    #[clap(long)]
    pub name: Option<String>,

    /// Filter sources that start by the given id
    #[clap(long)]
    pub id: Option<String>,
}

impl Input {
    fn matches(&self, s: &SourceAndTags) -> bool {
        match (&self.name, &self.id) {
            (Some(n), Some(i)) => s.source.abbrev.starts_with(n) && s.source.id.starts_with(i),
            (None, Some(i)) => s.source.id.starts_with(i),
            (Some(n), None) => s.source.abbrev.starts_with(n),
            (None, None) => true,
        }
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

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        let items = list_sources(args)
            .map(|r| r.items)
            .map_err(|source| CmdError::SourceList { source })?;
        let result = filter_sources(self, items);
        args.write_result(result)?;

        Ok(())
    }
}

fn filter_sources(args: &Input, sources: Vec<SourceAndTags>) -> Vec<SourceAndTags> {
    if args.name.is_none() && args.id.is_none() {
        sources
    } else {
        sources.into_iter().filter(|s| args.matches(s)).collect()
    }
}

pub fn list_sources(args: &CmdArgs) -> Result<SourceList, Error> {
    let url = &format!("{}/api/v1/sec/source", args.docspell_url());
    let client = reqwest::blocking::Client::new();
    let token = login::session_token(args).context(Login)?;
    client
        .get(url)
        .header(DOCSPELL_AUTH, token)
        .send()
        .and_then(|r| r.error_for_status())
        .context(Http { url })?
        .json::<SourceList>()
        .context(ReadResponse)
}
