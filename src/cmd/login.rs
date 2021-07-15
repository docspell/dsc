use std::{path::PathBuf, time::Duration};

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

    /// The password used for authentication in plain text. An
    /// environment variable DSC_PASSWORD can also be used.
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

    #[snafu(display("Invalid authentication token: {}", token))]
    InvalidAuthToken { token: String },

    #[snafu(display("Invalid password (non-unicode) in environment variable"))]
    InvalidPasswordEnv,
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
        None => match std::env::var_os(DSC_PASSWORD) {
            Some(pw) => {
                log::debug!("Using password from environment variable");
                pw.into_string().map_err(|_os| Error::InvalidPasswordEnv)
            }
            None => opts.password.clone().ok_or(Error::NoPassword),
        },
    }
}

fn get_account(opts: &Input, args: &CmdArgs) -> Result<String, Error> {
    let acc = match &opts.user {
        Some(u) => Ok(u.clone()),
        None => args.cfg.default_account.clone().ok_or(Error::NoAccount),
    };
    log::debug!("Using account: {:?}", &acc);
    acc
}

pub fn session_login(token: &str, args: &CmdArgs) -> Result<AuthResp, Error> {
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

    check_auth_result(result)
}

fn store_session(resp: &AuthResp) -> Result<(), Error> {
    match dirs::config_dir() {
        Some(mut dir) => {
            dir.push("dsc");
            dir.push(TOKEN_FILENAME);
            if !dir.exists() {
                log::debug!("Creating directory to store config at {:?}", dir.parent());
                std::fs::create_dir_all(dir.parent().unwrap())
                    .context(StoreSessionFile { path: dir.clone() })?;
            }
            log::debug!("Storing session to {}", dir.display());
            let cnt = serde_json::to_string(resp).context(SerializeSession)?;
            std::fs::write(&dir, &cnt).context(StoreSessionFile { path: dir })
        }
        None => Err(Error::NoSessionFile),
    }
}

/// Loads the session token from defined places. Uses in this order:
/// the option `--session`, the env variable `DSC_SESSION` or the
/// sesion file created by the `login` command.
///
/// If a session token can be loaded, it is checked for expiry and
/// refreshed if deemed necessary.
pub fn session_token(args: &CmdArgs) -> Result<String, Error> {
    let given_token = args
        .opts
        .session
        .clone()
        .or_else(|| get_token_from_env().clone());
    let no_token = given_token.is_none();
    let (token, valid) = match given_token {
        Some(token) => {
            log::debug!("Using auth token as given via option or env variable");
            Ok((token, None))
        }
        None => match dirs::config_dir() {
            Some(mut dir) => {
                dir.push("dsc");
                dir.push(TOKEN_FILENAME);
                let cnt = std::fs::read_to_string(&dir).context(ReadSessionFile { path: dir })?;
                let resp: AuthResp = serde_json::from_str(&cnt).context(SerializeSession)?;
                let token = get_token(&resp)?;
                Ok((token, Some(resp.valid_ms)))
            }
            None => Err(Error::NotLoggedIn),
        },
    }?;

    let created = extract_creation_time(&token)?;
    if near_expiry(created, valid) {
        log::info!("Token is nearly expired. Trying to refresh");
        let resp = session_login(&token, args)?;
        if no_token {
            store_session(&resp)?;
        } else {
            log::debug!("Not storing new session, since it was given as argument");
        }
        get_token(&resp)
    } else {
        Ok(token)
    }
}

fn near_expiry(created: u64, valid: Option<u64>) -> bool {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap();
    let created_ms = Duration::from_millis(created);
    let diff = now - created_ms;

    match valid {
        Some(valid_ms) => {
            let threshold = Duration::from_millis(((valid_ms as f64) * 0.8) as u64);
            log::debug!("Token age: {:?}  Threshold: {:?}", diff, threshold);
            diff.gt(&threshold)
        }
        None => {
            log::debug!("Token age: {:?}", diff);
            diff.gt(&Duration::from_secs(180))
        }
    }
}

fn get_token_from_env() -> Option<String> {
    std::env::var_os(DSC_SESSION)
        .filter(|s| !s.is_empty())
        .and_then(|s| s.into_string().ok())
}

fn extract_creation_time(token: &str) -> Result<u64, Error> {
    match token.split('-').next() {
        Some(ms) => ms.parse().map_err(|_e| Error::InvalidAuthToken {
            token: token.to_string(),
        }),
        None => Err(Error::InvalidAuthToken {
            token: token.to_string(),
        }),
    }
}

fn get_token(resp: &AuthResp) -> Result<String, Error> {
    match &resp.token {
        Some(t) => Ok(t.clone()),
        None => Err(Error::NotLoggedIn),
    }
}

fn check_auth_result(result: AuthResp) -> Result<AuthResp, Error> {
    if result.success {
        Ok(result)
    } else {
        log::debug!("Login result: {:?}", result);
        Err(Error::LoginFailed)
    }
}

const TOKEN_FILENAME: &'static str = "dsc-token.json";
const DSC_SESSION: &'static str = "DSC_SESSION";
const DSC_PASSWORD: &'static str = "DSC_PASSWORD";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_extract_creation_time() {
        let token =
            "1626345633653-ZGVtby9kZW1v-$2a$10$63d9R5xyDMYusXNdPdfKYO-e0jDd0o2KgBdrHv3PN+qTM+cFPM=";
        assert_eq!(extract_creation_time(token).unwrap(), 1626345633653);
    }
}
