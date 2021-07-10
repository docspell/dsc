use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::pass;
use crate::types::DOCSPELL_AUTH;
use clap::{ArgGroup, Clap};
use serde::{Deserialize, Serialize};

/// Performs a login given user credentials. The returned token is
/// stored on disk and used for subsequent calls to secured api
/// endpoints.
#[derive(Clap, std::fmt::Debug)]
#[clap(group = ArgGroup::new("pass"))]
pub struct Input {
    /// The account name.
    #[clap(long, short)]
    user: String,

    /// The password used for authentication in plain text.
    #[clap(long, group = "pass")]
    password: Option<String>,

    /// An entry for the pass password manager. If this is given, the
    /// `password` option is ignored.
    #[clap(long, group = "pass")]
    pass_entry: Option<String>,
}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        let result = login(self, args).and_then(|r| args.make_str(&r));
        println!("{:}", result?);
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

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResp {
    pub collective: String,
    pub user: String,
    pub success: bool,
    pub message: String,
    pub token: Option<String>,
    #[serde(alias = "validMs")]
    pub valid_ms: u64,
}

fn login(opts: &Input, args: &CmdArgs) -> Result<AuthResp, CmdError> {
    let url = format!("{}/api/v1/open/auth/login", args.docspell_url());
    let body = AuthRequest {
        account: opts.user.clone(),
        password: get_password(opts, args)?,
        remember_me: false,
    };
    let client = reqwest::blocking::Client::new();
    let result = client
        .post(url)
        .json(&body)
        .send()
        .map_err(CmdError::HttpError)?
        .json::<AuthResp>()
        .map_err(CmdError::HttpError)?;

    check_auth_result(result).and_then(|r| {
        store_session(&r)?;
        Ok(r)
    })
}

fn get_password(opts: &Input, args: &CmdArgs) -> Result<String, CmdError> {
    match args.pass_entry(&opts.pass_entry) {
        Some(pe) => pass::pass_password(&pe).map_err(CmdError::IOError),
        None => opts
            .password
            .clone()
            .ok_or(CmdError::InvalidInput("No password provided!".into())),
    }
}

fn session(token: &str, args: &CmdArgs) -> Result<AuthResp, CmdError> {
    let url = format!("{}/api/v1/sec/auth/session", args.docspell_url());
    let client = reqwest::blocking::Client::new();
    let result = client
        .post(url)
        .header(DOCSPELL_AUTH, token)
        .send()
        .map_err(CmdError::HttpError)?
        .json::<AuthResp>()
        .map_err(CmdError::HttpError)?;

    check_auth_result(result).and_then(|r| {
        store_session(&r)?;
        Ok(r)
    })
}

fn store_session(resp: &AuthResp) -> Result<(), CmdError> {
    match dirs::config_dir() {
        Some(mut dir) => {
            dir.push("dsc");
            dir.push(TOKEN_FILENAME);
            let cnt = serde_json::to_string(resp).map_err(CmdError::JsonSerError)?;
            std::fs::write(&dir, &cnt).map_err(CmdError::IOError)
        }
        None => Err(CmdError::AuthError("Cannot find token file".into())),
    }
}

pub fn session_token(args: &CmdArgs) -> Result<String, CmdError> {
    match dirs::config_dir() {
        Some(mut dir) => {
            dir.push("dsc");
            dir.push(TOKEN_FILENAME);
            let cnt = std::fs::read_to_string(&dir).map_err(CmdError::IOError)?;
            let resp: AuthResp =
                serde_json::from_str(&cnt).map_err(|e| CmdError::AuthError(e.to_string()))?;
            get_token(resp)
                .and_then(|t| session(&t, args))
                .and_then(|r| get_token(r))
        }
        None => Err(CmdError::AuthError("Not logged in.".into())),
    }
}

fn get_token(resp: AuthResp) -> Result<String, CmdError> {
    match resp.token {
        Some(t) => Ok(t),
        None => Err(CmdError::AuthError("Not logged in.".into())),
    }
}

fn check_auth_result(result: AuthResp) -> Result<AuthResp, CmdError> {
    if result.success {
        Ok(result)
    } else {
        Err(CmdError::AuthError(result.message))
    }
}

const TOKEN_FILENAME: &'static str = "dsc-token.json";
