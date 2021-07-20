pub mod get;

use crate::cmd::{Cmd, CmdArgs, CmdError};
use clap::{AppSettings, Clap};
use snafu::Snafu;

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
}

#[derive(Debug, Snafu)]
pub struct Error {}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        match &self.subcmd {
            ItemCommand::Get(input) => input.exec(args),
        }
    }
}
