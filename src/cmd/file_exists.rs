use crate::cmd::login;
use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::file;
use crate::opts::EndpointOpts;
use crate::types::{CheckFileResult, DOCSPELL_AUTH};
use crate::DsConfig;
use clap::Clap;
use reqwest::blocking::RequestBuilder;
use std::path::PathBuf;

/// Checks if the given files exist in docspell.
#[derive(Clap, Debug)]
pub struct Input {
    #[clap(flatten)]
    pub endpoint: EndpointOpts,

    /// Use the given source id.
    #[clap(long, group = "int")]
    pub source: Option<String>,

    /// One or more files to check
    #[clap(required = true, min_values = 1)]
    pub files: Vec<PathBuf>,
}
impl Input {
    fn collective_id(&self) -> Result<&String, CmdError> {
        self.endpoint
            .collective
            .as_ref()
            .ok_or(CmdError::InvalidInput(
                "Collective must be present when using integration endpoint.".into(),
            ))
    }

    fn source_id(&self, cfg: &DsConfig) -> Option<String> {
        self.source.clone().or(cfg.default_source_id.clone())
    }
}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        for file in &self.files {
            let result = check_file(&file, self, args).and_then(|r| args.make_str(&r));
            println!("{:}", result?);
        }

        Ok(())
    }
}

fn check_file(file: &PathBuf, args: &Input, opts: &CmdArgs) -> Result<CheckFileResult, CmdError> {
    let hash = file::digest_file_sha256(file).map_err(CmdError::IOError)?;
    let mut result = check_hash(&hash, args, opts)?;
    result.file = file.canonicalize().ok().map(|p| p.display().to_string());
    Ok(result)
}

fn check_hash(hash: &str, args: &Input, opts: &CmdArgs) -> Result<CheckFileResult, CmdError> {
    let url = if args.endpoint.integration {
        let coll_id = args.collective_id()?;
        format!(
            "{}/api/v1/open/integration/checkfile/{}/{}",
            opts.opts.docspell_url, coll_id, hash
        )
    } else {
        match args.source_id(opts.cfg) {
            Some(id) => format!(
                "{}/api/v1/open/checkfile/{}/{}",
                opts.opts.docspell_url, id, hash
            ),
            None => format!("{}/api/v1/sec/checkfile/{}", opts.opts.docspell_url, hash),
        }
    };

    let client = create_client(&url, args, opts)?;
    client
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(CmdError::HttpError)?
        .json::<CheckFileResult>()
        .map_err(CmdError::HttpError)
}

fn create_client(url: &str, args: &Input, cfg: &CmdArgs) -> Result<RequestBuilder, CmdError> {
    if args.source_id(cfg.cfg).is_none() && !args.endpoint.integration {
        let token = login::session_token(cfg.opts)?;
        Ok(reqwest::blocking::Client::new()
            .get(url)
            .header(DOCSPELL_AUTH, token))
    } else {
        let mut c = reqwest::blocking::Client::new().get(url);
        c = args.endpoint.apply(c);
        Ok(c)
    }
}

// fn int_endpoint_available(
//     args: &Input,
//     cfg: &ConfigOpts,
//     collective: &str,
// ) -> Result<reqwest::blocking::Response, CmdError> {
//     use reqwest::StatusCode;
//     let url = format!(
//         "{}/api/v1/open/integration/item/{}",
//         cfg.docspell_url, collective
//     );
//     log::debug!("Checking availability of integration endpoint: {}", url);
//     create_client(&url, args, cfg)?.send().map_or_else(
//         |e| Err(CmdError::HttpError(e)),
//         |r| match r.status() {
//             StatusCode::NOT_FOUND => Err(CmdError::IntEndpointNotAvail),
//             StatusCode::UNAUTHORIZED => Err(CmdError::AuthError(
//                 "Integration endpoint auth failed.".into(),
//             )),
//             StatusCode::FORBIDDEN => Err(CmdError::AuthError(
//                 "Integration endpoint auth failed.".into(),
//             )),
//             _ => r.error_for_status().map_err(CmdError::HttpError),
//         },
//     )
// }
