use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::types::VersionInfo;
use clap::Clap;

/// Queries the server for its version information.
#[derive(Clap, std::fmt::Debug)]
pub struct Input {}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        let result = version(args).and_then(|r| args.make_str(&r));
        println!("{:}", result?);
        Ok(())
    }
}

fn version(args: &CmdArgs) -> Result<VersionInfo, CmdError> {
    let url = format!("{}/api/info/version", args.docspell_url());
    return reqwest::blocking::get(url)
        .map_err(CmdError::HttpError)?
        .json::<VersionInfo>()
        .map_err(CmdError::HttpError);
}
