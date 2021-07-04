use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::opts::ConfigOpts;
use crate::types::BasicResult;
use crate::types::DOCSPELL_ADMIN;
use clap::Clap;

/// [Admin] Submits a task to generate preview images of all files.
/// This requires the admin secret from the server config file.
#[derive(Clap, std::fmt::Debug)]
pub struct Input {}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        let result = generate_previews(args.opts).and_then(|r| args.make_str(&r));
        println!("{:}", result?);
        Ok(())
    }
}

fn generate_previews(cfg: &ConfigOpts) -> Result<BasicResult, CmdError> {
    let url = format!(
        "{}/api/v1/admin/attachments/generatePreviews",
        cfg.docspell_url
    );
    let client = reqwest::blocking::Client::new();
    let secret = cfg
        .admin_secret
        .as_ref()
        .ok_or(CmdError::AuthError("No admin secret provided".into()))?;
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
