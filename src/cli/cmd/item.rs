pub mod get;
pub mod tags;

use clap::{AppSettings, Clap};
use snafu::{ResultExt, Snafu};

use super::{Cmd, Context};

/// Manage items.
#[derive(Clap, std::fmt::Debug)]
pub struct Input {
    #[clap(subcommand)]
    pub subcmd: ItemCommand,
}

#[derive(Clap, Debug)]
pub enum ItemCommand {
    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version)]
    Get(get::Input),

    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version)]
    Tags(tags::Input),
}

#[derive(Debug, Snafu)]
pub enum Error {
    Get { source: get::Error },
    Tags { source: tags::Error },
}

impl Cmd for Input {
    type CmdError = Error;

    fn exec(&self, ctx: &Context) -> Result<(), Error> {
        match &self.subcmd {
            ItemCommand::Get(input) => input.exec(ctx).context(Get),
            ItemCommand::Tags(input) => input.exec(ctx).context(Tags),
        }
    }
}
