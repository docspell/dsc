use crate::cmd::{admin, Cmd, CmdArgs, CmdError};
use crate::types::BasicResult;
use crate::types::DOCSPELL_ADMIN;
use clap::Clap;
use snafu::{ResultExt, Snafu};

/// Submits a task to re-create the entire fulltext search index.
#[derive(Clap, std::fmt::Debug)]
pub struct Input {}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        let result =
            recreate_index(args).map_err(|source| CmdError::AdminRecreateIndex { source })?;
        args.write_result(result)?;
        Ok(())
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error received from server at {}: {}", url, source))]
    Http { source: reqwest::Error, url: String },

    #[snafu(display("Error received from server: {}", source))]
    ReadResponse { source: reqwest::Error },

    #[snafu(display("No admin secret provided!"))]
    NoAdminSecret,
}

pub fn recreate_index(args: &CmdArgs) -> Result<BasicResult, Error> {
    let secret = admin::get_secret(args).ok_or(Error::NoAdminSecret)?;
    let url = &format!("{}/api/v1/admin/fts/reIndexAll", args.docspell_url());
    let client = reqwest::blocking::Client::new();
    client
        .post(url)
        .header(DOCSPELL_ADMIN, secret)
        .send()
        .and_then(|r| r.error_for_status())
        .context(Http { url })?
        .json::<BasicResult>()
        .context(ReadResponse)
}
