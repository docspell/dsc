pub mod list;

use clap::Parser;
use snafu::{ResultExt, Snafu};

use super::{Cmd, Context};

/// Manage source urls for uploading files.
#[derive(Parser, std::fmt::Debug)]
pub struct Input {
    #[clap(subcommand)]
    pub subcmd: SourceCommand,
}

#[derive(Parser, Debug)]
pub enum SourceCommand {
    #[command(version)]
    List(list::Input),
}

#[derive(Debug, Snafu)]
pub enum Error {
    List { source: list::Error },
}

impl Cmd for Input {
    type CmdError = Error;

    fn exec(&self, args: &Context) -> Result<(), Error> {
        match &self.subcmd {
            SourceCommand::List(input) => input.exec(args).context(ListSnafu),
        }
    }
}
