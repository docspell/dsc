//! Provides a library to use Docspell.
//!
//! The `http` module contains a client to Docspell. This is used in
//! the `cli` module to provide commands.

pub mod cli;
pub mod config;
pub mod error;
pub mod http;
mod util;

pub use cli::execute_cmd;

use clap::Clap;
use cli::opts::MainOpts;
use config::DsConfig;
use error::Result;
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
    execute_cmd(cfg, opts)?;
    Ok(())
}
