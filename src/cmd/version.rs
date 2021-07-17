use crate::types::AllVersion;
use crate::{
    cmd::{Cmd, CmdArgs, CmdError},
    types::VersionInfo,
};
use clap::Clap;
use reqwest::blocking::Client;
use snafu::{ResultExt, Snafu};

/// Prints version about server and client.
///
/// Queries the server for its version information and prints more
/// version details about this client.
#[derive(Clap, Debug, PartialEq)]
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
        let url = args.docspell_url();
        let result =
            version(url.as_str(), &args.client).map_err(|source| CmdError::Version { source })?;

        let vinfo = AllVersion::default(result, url);
        args.write_result(vinfo)?;

        Ok(())
    }
}

pub fn version(docspell_url: &str, client: &Client) -> Result<VersionInfo, Error> {
    let url = &format!("{}/api/info/version", docspell_url);
    return client
        .get(url)
        .send()
        .context(Http { url })?
        .json::<VersionInfo>()
        .context(SerializeResp {});
}
