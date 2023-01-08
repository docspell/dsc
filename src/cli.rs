//! Defines the command line interface.

pub mod cmd;
pub mod opts;
pub mod sink;
pub mod table;

use crate::config::DsConfig;
use clap::CommandFactory;

use self::cmd::{Cmd, CmdError, Context};
use self::opts::{MainOpts, SubCommand};

/// Given the config and arguments, runs the corresponding command.
pub fn execute_cmd(cfg: DsConfig, opts: MainOpts) -> Result<(), CmdError> {
    let ctx = Context::new(&opts.common_opts, &cfg)?;

    log::info!("Running command: {:?}", opts.subcmd);
    match &opts.subcmd {
        SubCommand::WriteDefaultConfig => {
            let cfg_file = DsConfig::write_default_file()?;
            eprintln!("Wrote config to {:}", cfg_file.display());
        }
        SubCommand::GenerateCompletions(input) => {
            let mut app = MainOpts::command();
            input.print_completions(&mut app);
        }
        SubCommand::Bookmark(input) => input.exec(&ctx)?,
        SubCommand::Item(input) => input.exec(&ctx)?,
        SubCommand::Watch(input) => input.exec(&ctx)?,
        SubCommand::Version(input) => input.exec(&ctx)?,
        SubCommand::Login(input) => input.exec(&ctx)?,
        SubCommand::Logout(input) => input.exec(&ctx)?,
        SubCommand::Search(input) => input.exec(&ctx)?,
        SubCommand::SearchSummary(input) => input.exec(&ctx)?,
        SubCommand::Source(input) => input.exec(&ctx)?,
        SubCommand::Admin(input) => input.exec(&ctx)?,
        SubCommand::FileExists(input) => input.exec(&ctx)?,
        SubCommand::GenInvite(input) => input.exec(&ctx)?,
        SubCommand::Register(input) => input.exec(&ctx)?,
        SubCommand::OpenItem(input) => input.exec(&ctx)?,
        SubCommand::Upload(input) => input.exec(&ctx)?,
        SubCommand::Download(input) => input.exec(&ctx)?,
        SubCommand::View(input) => input.exec(&ctx)?,
        SubCommand::Cleanup(input) => input.exec(&ctx)?,
        SubCommand::Export(input) => input.exec(&ctx)?,
    };
    Ok(())
}
