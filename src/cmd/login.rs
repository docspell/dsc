use std::path::PathBuf;

use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::pass;
use crate::types::{AuthResp, DOCSPELL_AUTH};
use clap::{ArgGroup, Clap};
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};

/// Performs a login given user credentials. The returned token is
/// stored on disk and used for subsequent calls to secured api
/// endpoints.
#[derive(Clap, std::fmt::Debug)]
#[clap(group = ArgGroup::new("pass"))]
pub struct Input {
    /// The account name. If not given here, it is looked up in the
    /// config file.
    #[clap(long, short)]
    user: Option<String>,

    /// The password used for authentication in plain text.
    #[clap(long, group = "pass")]
    password: Option<String>,

    /// An entry for the pass password manager. If this is given, the
    /// `password` option is ignored.
    #[clap(long, group = "pass")]
    pass_entry: Option<String>,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error received from server at {}: {}", url, source))]
    Http { source: reqwest::Error, url: String },

    #[snafu(display("Error received from server: {}", source))]
    ReadResponse { source: reqwest::Error },

    #[snafu(display("Retrieving password using pass failed: {}", source))]
    PassEntry { source: std::io::Error },

    #[snafu(display("No password provided!"))]
    NoPassword,

    #[snafu(display("No account name provided!"))]
    NoAccount,

    #[snafu(display("Error serializing auth response: {}", source))]
    SerializeSession { source: serde_json::Error },

    #[snafu(display("Error storing session file at {}: {}", path.display(), source))]
    StoreSessionFile {
        source: std::io::Error,
        path: PathBuf,
    },
    #[snafu(display("Error reading session file at {}: {}", path.display(), source))]
    ReadSessionFile {
        source: std::io::Error,
        path: PathBuf,
    },

    #[snafu(display("No session file found!"))]
    NoSessionFile,

    #[snafu(display("You are not logged in!"))]
    NotLoggedIn,

    #[snafu(display("Login failed!"))]
    LoginFailed,
}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        let result = login(self, args).map_err(|source| CmdError::Login { source })?;
        args.write_result(result)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthRequest {
    #[serde(alias = "account")]
    account: String,
    #[serde(alias = "password")]
    password: String,
    #[serde(alias = "rememberMe")]
    remember_me: bool,
}

pub fn login(opts: &Input, args: &CmdArgs) -> Result<AuthResp, Error> {
    let url = &format!("{}/api/v1/open/auth/login", args.docspell_url());
    let body = AuthRequest {
        account: get_account(opts, args)?,
        password: get_password(opts, args)?,
        remember_me: false,
    };
    let client = reqwest::blocking::Client::new();
    let result = client
        .post(url)
        .json(&body)
        .send()
        .and_then(|r| r.error_for_status())
        .context(Http { url })?
        .json::<AuthResp>()
        .context(ReadResponse)?;

    check_auth_result(result).and_then(|r| {
        store_session(&r)?;
        Ok(r)
    })
}

fn get_password(opts: &Input, args: &CmdArgs) -> Result<String, Error> {
    match args.pass_entry(&opts.pass_entry) {
        Some(pe) => pass::pass_password(&pe).context(PassEntry),
        None => opts.password.clone().ok_or(Error::NoPassword),
    }
}

fn get_account(opts: &Input, args: &CmdArgs) -> Result<String, Error> {
    match &opts.user {
        Some(u) => Ok(u.clone()),
        None => args.cfg.default_account.clone().ok_or(Error::NoAccount),
    }
}

pub fn session(token: &str, args: &CmdArgs) -> Result<AuthResp, Error> {
    let url = &format!("{}/api/v1/sec/auth/session", args.docspell_url());
    let client = reqwest::blocking::Client::new();
    let result = client
        .post(url)
        .header(DOCSPELL_AUTH, token)
        .send()
        .and_then(|r| r.error_for_status())
        .context(Http { url })?
        .json::<AuthResp>()
        .context(ReadResponse)?;

    check_auth_result(result).and_then(|r| {
        store_session(&r)?;
        Ok(r)
    })
}

fn store_session(resp: &AuthResp) -> Result<(), Error> {
    match dirs::config_dir() {
        Some(mut dir) => {
            dir.push("dsc");
            dir.push(TOKEN_FILENAME);
            let cnt = serde_json::to_string(resp).context(SerializeSession)?;
            std::fs::write(&dir, &cnt).context(StoreSessionFile { path: dir })
        }
        None => Err(Error::NoSessionFile),
    }
}

pub fn session_token(args: &CmdArgs) -> Result<String, Error> {
    match dirs::config_dir() {
        Some(mut dir) => {
            dir.push("dsc");
            dir.push(TOKEN_FILENAME);
            let cnt = std::fs::read_to_string(&dir).context(ReadSessionFile { path: dir })?;
            let resp: AuthResp = serde_json::from_str(&cnt).context(SerializeSession)?;
            get_token(resp)
                .and_then(|t| session(&t, args))
                .and_then(|r| get_token(r))
        }
        None => Err(Error::NotLoggedIn),
    }
}

fn get_token(resp: AuthResp) -> Result<String, Error> {
    match resp.token {
        Some(t) => Ok(t),
        None => Err(Error::NotLoggedIn),
    }
}

fn check_auth_result(result: AuthResp) -> Result<AuthResp, Error> {
    if result.success {
        Ok(result)
    } else {
        Err(Error::LoginFailed)
    }
}

const TOKEN_FILENAME: &'static str = "dsc-token.json";
