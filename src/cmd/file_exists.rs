use crate::cmd::login;
use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::file;
use crate::opts::ConfigOpts;
use crate::types::{CheckFileResult, DOCSPELL_AUTH};
use clap::{ArgGroup, Clap};
use reqwest::blocking::RequestBuilder;
use reqwest::StatusCode;
use std::path::PathBuf;

/// Checks if the given files exist in docspell.
#[derive(Clap, std::fmt::Debug)]
pub struct Input {
    #[clap(flatten)]
    pub endpoint: EndpointOpts,

    /// One or more files to check
    #[clap(required = true, min_values = 1)]
    pub files: Vec<PathBuf>,
}

#[derive(Clap, Debug)]
#[clap(group = ArgGroup::new("int"))]
pub struct EndpointOpts {
    /// Use the integration endpoint and provide the Basic auth header
    /// as credential. This must be a `username:password` pair.
    #[clap(long, group = "int")]
    basic: Option<NameVal>,

    /// Use the integration endpoint and provide the http header as
    /// credentials. This must be a `Header:Value` pair.
    #[clap(long, group = "int")]
    header: Option<NameVal>,

    /// Use the integration endpoint without any credentials.
    #[clap(long)]
    integration: bool,

    /// When using the integration endpoint, this is the collective to
    /// check against.
    #[clap(long)]
    collective: Option<String>,

    /// Use the given source id.
    #[clap(long, group = "int")]
    source: Option<String>,
}

#[derive(Debug)]
struct NameVal {
    name: String,
    value: String,
}

impl std::str::FromStr for NameVal {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pos = s
            .find(':')
            .ok_or_else(|| format!("Not a key:value pair, no `:` found in '{}'", s))?;
        Ok(NameVal {
            name: s[..pos].into(),
            value: s[pos + 1..].into(),
        })
    }
}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
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
    let url = if args.endpoint.integration {
        let coll_id = args
            .endpoint
            .collective
            .as_ref()
            .ok_or(CmdError::InvalidInput(
                "Collective must be present when using integration endpoint.".into(),
            ))?;
        let u = format!(
            "{}/api/v1/open/integration/checkfile/{}/{}",
            cfg.docspell_url, coll_id, hash
        );
        int_endpoint_available(args, cfg, coll_id)?;
        u
    } else {
        match &args.endpoint.source {
            Some(id) => format!("{}/api/v1/open/checkfile/{}/{}", cfg.docspell_url, id, hash),
            None => format!("{}/api/v1/sec/checkfile/{}", cfg.docspell_url, hash),
        }
    };

    let client = create_client(&url, args, cfg)?;
    client
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(CmdError::HttpError)?
        .json::<CheckFileResult>()
        .map_err(CmdError::HttpError)
}

fn create_client(url: &str, args: &Input, cfg: &ConfigOpts) -> Result<RequestBuilder, CmdError> {
    if args.endpoint.source.is_none() && !args.endpoint.integration {
        let token = login::session_token(cfg)?;
        Ok(reqwest::blocking::Client::new()
            .get(url)
            .header(DOCSPELL_AUTH, token))
    } else {
        let mut c = reqwest::blocking::Client::new().get(url);
        if let Some(h) = &args.endpoint.header {
            c = apply_int_header(h, c)
        }
        if let Some(b) = &args.endpoint.basic {
            c = apply_int_basic(b, c)
        }
        Ok(c)
    }
}
fn apply_int_basic(b: &NameVal, c: RequestBuilder) -> RequestBuilder {
    log::debug!(
        "Using integration endpoint with basic auth: {}:{}",
        b.name,
        b.value
    );
    c.basic_auth(&b.name, Some(&b.value))
}
fn apply_int_header(h: &NameVal, c: RequestBuilder) -> RequestBuilder {
    log::debug!(
        "Using integration endpoint with header: {}:{}",
        h.name,
        h.value
    );
    c.header(&h.name, &h.value)
}

fn int_endpoint_available(
    args: &Input,
    cfg: &ConfigOpts,
    collective: &str,
) -> Result<reqwest::blocking::Response, CmdError> {
    let url = format!(
        "{}/api/v1/open/integration/item/{}",
        cfg.docspell_url, collective
    );
    log::debug!("Checking availability of integration endpoint: {}", url);
    create_client(&url, args, cfg)?.send().map_or_else(
        |e| Err(CmdError::HttpError(e)),
        |r| match r.status() {
            StatusCode::NOT_FOUND => Err(CmdError::IntEndpointNotAvail),
            StatusCode::UNAUTHORIZED => Err(CmdError::AuthError(
                "Integration endpoint auth failed.".into(),
            )),
            StatusCode::FORBIDDEN => Err(CmdError::AuthError(
                "Integration endpoint auth failed.".into(),
            )),
            _ => r.error_for_status().map_err(CmdError::HttpError),
        },
    )
}
