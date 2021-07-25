pub mod fields;
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

    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version)]
    Fields(fields::Input),
}

#[derive(Debug, Snafu)]
pub enum Error {
    Get { source: get::Error },
    Tags { source: tags::Error },
    Fields { source: fields::Error },
}

impl Cmd for Input {
    type CmdError = Error;

    fn exec(&self, ctx: &Context) -> Result<(), Error> {
        match &self.subcmd {
            ItemCommand::Get(input) => input.exec(ctx).context(Get),
            ItemCommand::Tags(input) => input.exec(ctx).context(Tags),
            ItemCommand::Fields(input) => input.exec(ctx).context(Fields),
        }
    }
}
