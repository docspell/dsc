use crate::cmd::login;
use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::types::{SourceAndTags, SourceList, DOCSPELL_AUTH};
use clap::Clap;

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

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        let items = list_sources(args).map(|r| r.items)?;
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

fn list_sources(args: &CmdArgs) -> Result<SourceList, CmdError> {
    let url = format!("{}/api/v1/sec/source", args.docspell_url());
    let client = reqwest::blocking::Client::new();
    let token = login::session_token(args)?;
    client
        .get(url)
        .header(DOCSPELL_AUTH, token)
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(CmdError::HttpError)?
        .json::<SourceList>()
        .map_err(CmdError::HttpError)
}
