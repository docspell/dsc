pub mod get;

use clap::Parser;
use snafu::{ResultExt, Snafu};

use super::{Cmd, Context};

/// Manage bookmarks.
#[derive(Parser, std::fmt::Debug)]
pub struct Input {
    #[clap(subcommand)]
    pub subcmd: BookmarkCommand,
}

#[derive(Parser, Debug)]
pub enum BookmarkCommand {
    #[clap(version)]
    Get(get::Input),
}

#[derive(Debug, Snafu)]
pub enum Error {
    Get { source: get::Error },
}

impl Cmd for Input {
    type CmdError = Error;

    fn exec(&self, ctx: &Context) -> Result<(), Error> {
        match &self.subcmd {
            BookmarkCommand::Get(input) => input.exec(ctx).context(GetSnafu),
        }
    }
}
