pub mod cmd;
pub mod config;
pub mod opts;

use clap::Clap;
use config::{ConfigError, DsConfig};
use log;
use opts::MainOpts;

pub fn read_args() -> MainOpts {
    let m = MainOpts::parse();
    log::debug!("Parsed options: {:?}", m);
    m
}

pub fn read_config(file: &Option<String>) -> Result<DsConfig, ConfigError> {
    let f = DsConfig::read(file);
    log::debug!("Read config file: {:?}", f);
    f
}
