use crate::cmd::login;
use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::file;
use crate::opts::EndpointOpts;
use crate::types::{CheckFileResult, DOCSPELL_AUTH};
use crate::DsConfig;
use clap::Clap;
use reqwest::blocking::RequestBuilder;
use snafu::{ResultExt, Snafu};
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
    fn collective_id(&self) -> Result<&String, Error> {
        self.endpoint.collective.as_ref().ok_or(Error::NoCollective)
    }

    fn source_id(&self, cfg: &DsConfig) -> Option<String> {
        self.source.clone().or(cfg.default_source_id.clone())
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Collective must be present when using integration endpoint."))]
    NoCollective,

    #[snafu(display("Calculating digest of file {} failed: {}", path.display(), source))]
    DigestFail {
        source: std::io::Error,
        path: PathBuf,
    },

    #[snafu(display("Error received from server at {}: {}", url, source))]
    Http { source: reqwest::Error, url: String },

    #[snafu(display("Error received from server: {}", source))]
    ReadResponse { source: reqwest::Error },

    #[snafu(display(
        "Error logging in via session. Consider the `login` command. {}",
        source
    ))]
    Login { source: login::Error },
}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        let mut results = Vec::with_capacity(self.files.capacity());
        for file in &self.files {
            let result =
                check_file(&file, self, args).map_err(|source| CmdError::FileExists { source })?;
            results.push(result);
        }
        args.write_result(results)?;
        Ok(())
    }
}

pub fn check_file(file: &PathBuf, args: &Input, opts: &CmdArgs) -> Result<CheckFileResult, Error> {
    let hash = file::digest_file_sha256(file).context(DigestFail { path: file })?;
    let mut result = check_hash(&hash, args, opts)?;
    result.file = file.canonicalize().ok().map(|p| p.display().to_string());
    Ok(result)
}

pub fn check_hash(hash: &str, opts: &Input, args: &CmdArgs) -> Result<CheckFileResult, Error> {
    let url = &if opts.endpoint.integration {
        let coll_id = opts.collective_id()?;
        format!(
            "{}/api/v1/open/integration/checkfile/{}/{}",
            args.docspell_url(),
            coll_id,
            hash
        )
    } else {
        match opts.source_id(args.cfg) {
            Some(id) => format!(
                "{}/api/v1/open/checkfile/{}/{}",
                args.docspell_url(),
                id,
                hash
            ),
            None => format!("{}/api/v1/sec/checkfile/{}", args.docspell_url(), hash),
        }
    };

    let client = create_client(&url, opts, args)?;
    client
        .send()
        .and_then(|r| r.error_for_status())
        .context(Http { url })?
        .json::<CheckFileResult>()
        .context(ReadResponse)
}

fn create_client(url: &str, opts: &Input, args: &CmdArgs) -> Result<RequestBuilder, Error> {
    if opts.source_id(args.cfg).is_none() && !opts.endpoint.integration {
        let token = login::session_token(args).context(Login)?;
        Ok(reqwest::blocking::Client::new()
            .get(url)
            .header(DOCSPELL_AUTH, token))
    } else {
        let mut c = reqwest::blocking::Client::new().get(url);
        c = opts.endpoint.apply(c);
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
