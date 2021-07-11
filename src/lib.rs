pub mod cmd;
pub mod config;
mod file;
pub mod opts;
mod pass;
mod sink;
pub mod types;

use clap::Clap;
use cmd::{Cmd, CmdArgs};
use config::DsConfig;
use log;
use opts::{MainOpts, SubCommand};
use std::convert::From;
use std::path::PathBuf;

#[derive(Debug)]
pub enum DscError {
    Cmd(cmd::CmdError),
    Config(config::ConfigError),
}
impl From<cmd::CmdError> for DscError {
    fn from(e: cmd::CmdError) -> DscError {
        DscError::Cmd(e)
    }
}
impl From<config::ConfigError> for DscError {
    fn from(e: config::ConfigError) -> DscError {
        DscError::Config(e)
    }
}

pub fn read_args() -> MainOpts {
    let m = MainOpts::parse();
    log::debug!("Parsed options: {:?}", m);
    m
}

pub fn read_config(file: &Option<PathBuf>) -> Result<DsConfig, DscError> {
    let f = DsConfig::read(file.as_ref());
    log::debug!("Config: {:?}", f);
    f.map_err(DscError::Config)
}

pub fn execute() -> Result<(), DscError> {
    let opts = read_args();
    let cfg = read_config(&opts.config)?;
    eprintln!("Docspell at: {:}", cfg.docspell_url);
    execute_cmd(cfg, opts)
}

pub fn execute_cmd(cfg: DsConfig, opts: MainOpts) -> Result<(), DscError> {
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
    };
    Ok(())
}
