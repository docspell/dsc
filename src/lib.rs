//! Provides a library and command line interface to Docspell.
//!

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

/// Reads the program arguments into the `MainOpts` data structure.
pub fn read_args() -> MainOpts {
    log::debug!("Parsing command line options…");
    let m = MainOpts::parse();

    log::debug!("Parsed options: {:?}", m);
    m
}

/// Reads the config file.
///
/// If the file is not given, it is searched in the default location.
/// If the file is given, it is used without a fallback.
pub fn read_config(file: &Option<PathBuf>) -> Result<DsConfig> {
    let f = DsConfig::read(file.as_ref())?;
    log::debug!("Config: {:?}", f);
    Ok(f)
}

/// The main method: reads arguments and config file and executes the
/// corresponding command.
pub fn execute() -> Result<()> {
    let opts = read_args();
    let cfg = read_config(&opts.config)?;
    execute_cmd(cfg, opts)?;
    Ok(())
}
