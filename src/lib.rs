pub mod cmd;
pub mod config;
pub mod error;
mod file;
pub mod opts;
mod pass;
mod sink;
mod table;
pub mod types;

use clap::Clap;
use cmd::{Cmd, CmdArgs};
use config::DsConfig;
use error::Result;
use log;
use opts::{MainOpts, SubCommand};
use std::path::PathBuf;

pub fn read_args() -> MainOpts {
    log::debug!("Parsing command line options…");
    let m = MainOpts::parse();
    log::debug!("Parsed options: {:?}", m);
    m
}

pub fn read_config(file: &Option<PathBuf>) -> Result<DsConfig> {
    let f = DsConfig::read(file.as_ref())?;
    log::debug!("Config: {:?}", f);
    Ok(f)
}

pub fn execute() -> Result<()> {
    let opts = read_args();
    let cfg = read_config(&opts.config)?;
    execute_cmd(cfg, opts)
}

pub fn execute_cmd(cfg: DsConfig, opts: MainOpts) -> Result<()> {
    let args = CmdArgs {
        opts: &opts.common_opts,
        cfg: &cfg,
    };
    log::info!("Running command: {:?}", opts.subcmd);
    match &opts.subcmd {
        SubCommand::WriteDefaultConfig => {
            let cfg_file = DsConfig::write_default_file()?;
            eprintln!("Wrote config to {:}", cfg_file.display());
        }
        SubCommand::Version(input) => input.exec(&args)?,
        SubCommand::Login(input) => input.exec(&args)?,
        SubCommand::Search(input) => input.exec(&args)?,
        SubCommand::SearchSummary(input) => input.exec(&args)?,
        SubCommand::Source(input) => input.exec(&args)?,
        SubCommand::Admin(input) => input.exec(&args)?,
        SubCommand::FileExists(input) => input.exec(&args)?,
        SubCommand::GenInvite(input) => input.exec(&args)?,
        SubCommand::Register(input) => input.exec(&args)?,
        SubCommand::Upload(input) => input.exec(&args)?,
        SubCommand::Download(input) => input.exec(&args)?,
        SubCommand::View(input) => input.exec(&args)?,
    };
    Ok(())
}
