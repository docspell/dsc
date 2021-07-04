pub mod cmd;
pub mod config;
pub mod opts;
pub mod types;

use clap::Clap;
use cmd::{Cmd, CmdArgs};
use config::DsConfig;
use log;
use opts::{MainOpts, SubCommand};

#[derive(Debug)]
pub enum DscError {
    Cmd(cmd::CmdError),
    Config(config::ConfigError),
}

pub fn read_args() -> MainOpts {
    let m = MainOpts::parse();
    log::debug!("Parsed options: {:?}", m);
    m
}

pub fn read_config(file: &Option<String>) -> Result<DsConfig, DscError> {
    let f = DsConfig::read(file);
    log::debug!("Read config file: {:?}", f);
    f.map_err(DscError::Config)
}

pub fn execute() -> Result<(), DscError> {
    let opts = read_args();
    let cfg = read_config(&opts.config);
    cfg.and_then(|c| execute_cmd(&c, &opts))
}

pub fn execute_cmd(cfg: &DsConfig, opts: &MainOpts) -> Result<(), DscError> {
    let args = CmdArgs {
        cfg: cfg,
        opts: &opts.common_opts,
    };
    log::info!("Running command: {:?}", opts.subcmd);
    match &opts.subcmd {
        SubCommand::Version(input) => input.exec(&args).map_err(DscError::Cmd),
        SubCommand::Login(input) => input.exec(&args).map_err(DscError::Cmd),
        SubCommand::Search(input) => input.exec(&args).map_err(DscError::Cmd),
        SubCommand::SearchSummary(input) => input.exec(&args).map_err(DscError::Cmd),
    }
}
