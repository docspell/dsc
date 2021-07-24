//! Provides helpers to handle docspell "sessions".
//!
//! Docspell returns an authentication token for a login via account +
//! password. This token must be used for all secured endpoints.
//!
//! This token is stored on disk and also refreshed if it is almost
//! expired.
//!
//! This is for internal use only.

use snafu::{ResultExt, Snafu};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use super::payload::AuthResp;
use super::Client;

const TOKEN_FILENAME: &str = "dsc-token.json";
const DSC_SESSION: &str = "DSC_SESSION";

#[derive(Debug, Snafu)]
pub enum Error {
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

    #[snafu(display("Error storing session file at {}: {}", path.display(), source))]
    DeleteSessionFile {
        source: std::io::Error,
        path: PathBuf,
    },

    #[snafu(display("You are not logged in!"))]
    NotLoggedIn,

    #[snafu(display("Invalid authentication token: {}", token))]
    InvalidAuthToken { token: String },

    #[snafu(display("Error serializing auth response: {}", source))]
    SerializeSession { source: serde_json::Error },

    #[snafu(display("Error refreshing session. Use the `login` command. {}", mesg))]
    RefreshSession { mesg: String },
}

pub fn store_session(resp: &AuthResp) -> Result<(), Error> {
    match dirs::config_dir() {
        Some(mut dir) => {
            dir.push("dsc");
            dir.push(TOKEN_FILENAME);
            if !dir.exists() {
                log::debug!("Creating directory to store config at {:?}", dir.parent());
                std::fs::create_dir_all(dir.parent().unwrap())
                    .context(StoreSessionFile { path: dir.clone() })?;
            }
            write_token_file(resp, &dir)
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
pub fn session_token(token: &Option<String>, client: &Client) -> Result<String, Error> {
    let given_token = token.clone().or_else(get_token_from_env);
    let no_token = given_token.is_none();
    let (token, valid) = match given_token {
        Some(token) => {
            log::debug!("Using auth token as given via option or env variable");
            Ok((token, None))
        }
        None => {
            let file = get_token_file().map_err(|_err| Error::NotLoggedIn)?;
            let resp = read_token_file(&file)?;
            let token = get_token(&resp)?;
            Ok((token, Some(resp.valid_ms)))
        }
    }?;

    let created = extract_creation_time(&token)?;
    if near_expiry(created, valid) {
        log::info!("Token is nearly expired. Trying to refresh");
        let resp = client
            .session_login(&token)
            .map_err(|err| Error::RefreshSession {
                mesg: err.to_string(),
            })?;
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

pub fn drop_session() -> Result<(), Error> {
    let path = get_token_file()?;
    if path.exists() {
        std::fs::remove_file(&path).context(DeleteSessionFile { path })?;
    }
    Ok(())
}

// --- helper

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

fn get_token_from_env() -> Option<String> {
    std::env::var_os(DSC_SESSION)
        .filter(|s| !s.is_empty())
        .and_then(|s| s.into_string().ok())
}

fn get_token_file() -> Result<PathBuf, Error> {
    match dirs::config_dir() {
        Some(mut dir) => {
            dir.push("dsc");
            dir.push(TOKEN_FILENAME);
            Ok(dir)
        }
        None => Err(Error::NoSessionFile),
    }
}

fn read_token_file(path: &Path) -> Result<AuthResp, Error> {
    let _flock = acquire_lock(path, false)?;

    let cnt = std::fs::read_to_string(&path).context(ReadSessionFile { path })?;
    let resp: AuthResp = serde_json::from_str(&cnt).context(SerializeSession)?;
    Ok(resp)
}

fn write_token_file(resp: &AuthResp, path: &Path) -> Result<(), Error> {
    let flock = acquire_lock(path, true);
    match flock {
        Ok(_fl) => {
            log::debug!("Storing session to {}", path.display());
            let cnt = serde_json::to_string(resp).context(SerializeSession)?;
            std::fs::write(path, &cnt).context(StoreSessionFile { path })
        }
        Err(err) => {
            log::debug!(
                "Could not obtain write lock to store session in file: {}",
                err
            );
            Ok(())
        }
    }
}

fn get_token(resp: &AuthResp) -> Result<String, Error> {
    match &resp.token {
        Some(t) => Ok(t.clone()),
        None => Err(Error::NotLoggedIn),
    }
}

// --- file lock

#[cfg(windows)]
fn acquire_lock(path: &Path, write: bool) -> Result<(), Error> {
    Ok(())
}

#[cfg(unix)]
fn acquire_lock(path: &Path, write: bool) -> Result<(), Error> {
    if write {
        file_locker::FileLock::new(path)
            .blocking(false)
            .writeable(true)
            .lock()
            .map(|_fl| ())
            .context(StoreSessionFile { path })
    } else {
        file_locker::FileLock::new(path)
            .blocking(true)
            .writeable(false)
            .lock()
            .map(|_fl| ())
            .context(ReadSessionFile { path })
    }
}

// --- tests

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
