use crate::cmd::admin::AdminCmd;
use crate::cmd::{CmdArgs, CmdError};
use crate::types::BasicResult;
use crate::types::DOCSPELL_ADMIN;
use clap::Clap;

/// Submits a task to generate preview images of all files.
#[derive(Clap, std::fmt::Debug)]
pub struct Input {}

impl AdminCmd for Input {
    fn exec(&self, secret: &str, args: &CmdArgs) -> Result<(), CmdError> {
        let result = generate_previews(secret, args)?;
        args.write_result(result)?;
        Ok(())
    }
}

fn generate_previews(secret: &str, args: &CmdArgs) -> Result<BasicResult, CmdError> {
    let url = format!(
        "{}/api/v1/admin/attachments/generatePreviews",
        args.docspell_url()
    );
    let client = reqwest::blocking::Client::new();
    log::debug!("Using secret: {:}", secret);
    client
        .post(url)
        .header(DOCSPELL_ADMIN, secret)
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(CmdError::HttpError)?
        .json::<BasicResult>()
        .map_err(CmdError::HttpError)
}
