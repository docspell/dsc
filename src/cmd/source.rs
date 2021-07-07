pub mod list;

use crate::cmd::{Cmd, CmdArgs, CmdError};
use clap::{AppSettings, Clap};

/// Manage source urls for uploading files.
#[derive(Clap, std::fmt::Debug)]
pub struct Input {
    #[clap(subcommand)]
    pub subcmd: SourceCommand,
}

#[derive(Clap, Debug)]
pub enum SourceCommand {
    #[clap(setting = AppSettings::ColoredHelp)]
    #[clap(version)]
    List(list::Input),
}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        match &self.subcmd {
            SourceCommand::List(input) => input.exec(args),
        }
    }
}
