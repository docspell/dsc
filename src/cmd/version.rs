use crate::types::AllVersion;
use crate::{
    cmd::{Cmd, CmdArgs, CmdError},
    types::VersionInfo,
};
use clap::Clap;
use snafu::{ResultExt, Snafu};

/// Queries the server for its version information and prints more
/// version details about this client.
#[derive(Clap, std::fmt::Debug)]
pub struct Input {}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An error was received from: {}!", url))]
    Http { source: reqwest::Error, url: String },

    #[snafu(display("An error occured serializing the response!"))]
    SerializeResp { source: reqwest::Error },
}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        let result =
            version(args.docspell_url().as_str()).map_err(|source| CmdError::Version { source })?;

        let vinfo = AllVersion::default(result);
        args.write_result(vinfo)?;

        Ok(())
    }
}

pub fn version(docspell_url: &str) -> Result<VersionInfo, Error> {
    let url = &format!("{}/api/info/version", docspell_url);
    let client = reqwest::blocking::Client::new();
    return client
        .get(url)
        .send()
        .context(Http { url })?
        .json::<VersionInfo>()
        .context(SerializeResp {});
}
