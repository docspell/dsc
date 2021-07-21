pub mod cmd;
pub mod opts;
pub mod sink;
pub mod table;

use crate::config::DsConfig;
use clap::IntoApp;

use self::cmd::{Cmd, CmdError, Context};
use self::opts::{MainOpts, SubCommand};

pub fn execute_cmd(cfg: DsConfig, opts: MainOpts) -> Result<(), CmdError> {
    let ctx = Context::new(&opts.common_opts, &cfg);

    log::info!("Running command: {:?}", opts.subcmd);
    match &opts.subcmd {
        SubCommand::WriteDefaultConfig => {
            let cfg_file = DsConfig::write_default_file()?;
            eprintln!("Wrote config to {:}", cfg_file.display());
        }
        SubCommand::GenerateCompletions(input) => {
            let mut app = MainOpts::into_app();
            input.print_completions(&mut app);
        }
        // SubCommand::Item(input) => input.exec(&args)?,
        // SubCommand::Watch(input) => input.exec(&args)?,
        SubCommand::Version(input) => input.exec(&ctx)?,
        SubCommand::Login(input) => input.exec(&ctx)?,
        SubCommand::Logout(input) => input.exec(&ctx)?,
        SubCommand::Search(input) => input.exec(&ctx)?,
        // SubCommand::SearchSummary(input) => input.exec(&args)?,
        // SubCommand::Source(input) => input.exec(&args)?,
        // SubCommand::Admin(input) => input.exec(&args)?,
        // SubCommand::FileExists(input) => input.exec(&args)?,
        // SubCommand::GenInvite(input) => input.exec(&args)?,
        // SubCommand::Register(input) => input.exec(&args)?,
        // SubCommand::Upload(input) => input.exec(&args)?,
        // SubCommand::Download(input) => input.exec(&args)?,
        // SubCommand::View(input) => input.exec(&args)?,
        // SubCommand::Cleanup(input) => input.exec(&args)?,
    };
    Ok(())
}
