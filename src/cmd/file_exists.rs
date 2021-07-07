use crate::cmd::login;
use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::file;
use crate::opts::ConfigOpts;
use crate::types::{CheckFileResult, DOCSPELL_AUTH};
use clap::Clap;
use std::path::PathBuf;

/// Checks if the given files exist in docspell.
#[derive(Clap, std::fmt::Debug)]
pub struct Input {
    /// Specify an optional source id. If not given, login is required.
    #[clap(long, short)]
    pub source: Option<String>,

    /// One or more files to check
    #[clap(required = true, min_values = 1)]
    pub files: Vec<PathBuf>,
}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        // let result = check_file(&self.file, self, args.opts).and_then(|r| args.make_str(&r));
        // println!("{:}", result?);
        for file in &self.files {
            let result = check_file(&file, self, args.opts).and_then(|r| args.make_str(&r));
            println!("{:}", result?);
        }
        Ok(())
    }
}

fn check_file(file: &PathBuf, args: &Input, cfg: &ConfigOpts) -> Result<CheckFileResult, CmdError> {
    let hash = file::digest_file_sha256(file).map_err(CmdError::IOError)?;
    let mut result = check_hash(&hash, args, cfg)?;
    result.file = file.canonicalize().ok().map(|p| p.display().to_string());
    Ok(result)
}

fn check_hash(hash: &str, args: &Input, cfg: &ConfigOpts) -> Result<CheckFileResult, CmdError> {
    let url = match &args.source {
        Some(id) => format!("{}/api/v1/open/checkfile/{}/{}", cfg.docspell_url, id, hash),
        None => format!("{}/api/v1/sec/checkfile/{}", cfg.docspell_url, hash),
    };
    let client = if args.source.is_none() {
        let token = login::session_token(cfg)?;
        reqwest::blocking::Client::new()
            .get(url)
            .header(DOCSPELL_AUTH, token)
    } else {
        reqwest::blocking::Client::new().get(url)
    };

    client
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(CmdError::HttpError)?
        .json::<CheckFileResult>()
        .map_err(CmdError::HttpError)
}
