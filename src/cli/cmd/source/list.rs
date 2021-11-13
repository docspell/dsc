use clap::Parser;
use snafu::{ResultExt, Snafu};

use super::{Cmd, Context};
use crate::cli::sink::Error as SinkError;
use crate::http::payload::SourceAndTags;
use crate::http::Error as HttpError;

/// List all sources for your collective
#[derive(Parser, std::fmt::Debug)]
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
    #[snafu(display("An http error occurred: {}!", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },
}

impl Cmd for Input {
    type CmdError = Error;

    fn exec(&self, ctx: &Context) -> Result<(), Error> {
        let items = ctx
            .client
            .list_sources(&ctx.opts.session)
            .map(|r| r.items)
            .context(HttpClient)?;
        let result = filter_sources(self, items);
        ctx.write_result(result).context(WriteResult)?;
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
