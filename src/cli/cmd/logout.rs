use crate::{
    cmd::{login, Cmd, CmdArgs, CmdError},
    types::BasicResult,
};
use clap::Clap;
use snafu::{ResultExt, Snafu};
use std::path::PathBuf;

/// Removes the credentials file
#[derive(Clap, Debug)]
pub struct Input {}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error storing session file at {}: {}", path.display(), source))]
    DeleteSessionFile {
        source: std::io::Error,
        path: PathBuf,
    },

    #[snafu(display("No session file found!"))]
    NoSessionFile,
}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        remove_session_file().map_err(|source| CmdError::Logout { source })?;
        let message = BasicResult {
            success: true,
            message: "Session deleted.".into(),
        };
        args.write_result(message)?;
        Ok(())
    }
}

pub fn remove_session_file() -> Result<(), Error> {
    let path = login::get_token_file().map_err(|_err| Error::NoSessionFile)?;
    if path.exists() {
        std::fs::remove_file(&path).context(DeleteSessionFile { path })?;
    }
    Ok(())
}
