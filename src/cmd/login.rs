use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::config::DsConfig;
use clap::Clap;
use serde::{Deserialize, Serialize};

#[derive(Clap, std::fmt::Debug)]
pub struct Input {
    #[clap(long, short)]
    user: String,
    #[clap(long)]
    password: String,
    #[clap(long)]
    pass_entry: Option<String>,
}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        let result = login(self, args.cfg).and_then(|r| args.make_str(&r));
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

fn login(args: &Input, cfg: &DsConfig) -> Result<AuthResp, CmdError> {
    let url = format!("{}/api/v1/open/auth/login", cfg.docspell_url);
    let body = AuthRequest {
        account: args.user.clone(),
        password: args.password.clone(),
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

fn session(token: &str, cfg: &DsConfig) -> Result<AuthResp, CmdError> {
    let url = format!("{}/api/v1/open/auth/session", cfg.docspell_url);
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

pub fn session_token() -> Result<String, CmdError> {
    match dirs::config_dir() {
        Some(mut dir) => {
            dir.push("dsc");
            dir.push(TOKEN_FILENAME);
            let cnt = std::fs::read_to_string(&dir).map_err(CmdError::IOError)?;
            let resp: AuthResp =
                serde_json::from_str(&cnt).map_err(|e| CmdError::AuthError(e.to_string()))?;
            match resp.token {
                Some(t) => Ok(t),
                None => Err(CmdError::AuthError("Not logged in.".into())),
            }
        }
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

pub const DOCSPELL_AUTH: &'static str = "X-Docspell-Auth";
const TOKEN_FILENAME: &'static str = "dsc-token.json";
