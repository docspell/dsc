use crate::cmd::admin::AdminCmd;
use crate::cmd::{CmdArgs, CmdError};
use crate::opts::ConfigOpts;
use crate::types::BasicResult;
use crate::types::DOCSPELL_ADMIN;
use clap::Clap;

/// Submits a task to generate preview images of all files. This
/// requires the admin secret from the server config file.
#[derive(Clap, std::fmt::Debug)]
pub struct Input {}

impl AdminCmd for Input {
    fn exec(&self, secret: &str, args: &CmdArgs) -> Result<(), CmdError> {
        let result = generate_previews(secret, args.opts).and_then(|r| args.make_str(&r));
        println!("{:}", result?);
        Ok(())
    }
}

fn generate_previews(secret: &str, cfg: &ConfigOpts) -> Result<BasicResult, CmdError> {
    let url = format!(
        "{}/api/v1/admin/attachments/generatePreviews",
        cfg.docspell_url
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
