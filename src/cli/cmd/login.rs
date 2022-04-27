use super::{Cmd, Context};
use crate::cli::sink::Error as SinkError;
use crate::http::payload::{AuthRequest, AuthResp};
use crate::http::Error as HttpError;

use crate::util::pass;

use clap::{ArgGroup, Parser, ValueHint};
use rsotp::TOTP;
use snafu::{ResultExt, Snafu};
use std::io::Write;

/// Performs a login given user credentials.
///
/// The returned token is stored on disk in a session file and used
/// for subsequent calls to secured api endpoints. If the token is
/// near to expire, it is refreshed and the session file is updated.
///
/// It is also possible to specfiy a session token instead. When a
/// session token is given via options or env variable, the session
/// file is not updated (no filesystem access occurs).
#[derive(Parser, Debug, PartialEq)]
#[clap(group = ArgGroup::new("pass"))]
pub struct Input {
    /// The account name. If not given here, it is looked up in the
    /// config file.
    #[clap(long, short, value_hint = ValueHint::Username)]
    user: Option<String>,

    /// The password used for authentication in plain text. An
    /// environment variable DSC_PASSWORD can also be used.
    #[clap(long, group = "pass")]
    password: Option<String>,

    /// An entry for the pass password manager. If this is given, the
    /// `password` option is ignored.
    #[clap(long, group = "pass")]
    pass_entry: Option<String>,

    /// An entry for the pass password manager that contains the TOTP
    /// secret, so dsc can obtain the TOTP code automatically. If
    /// prefixed with `key:` the remaining part is looked up in the
    /// other `pass_entry` instead.
    #[clap(long)]
    pass_otp: Option<String>,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An http error occurred: {}", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error received from server: {}", source))]
    ReadResponse { source: reqwest::Error },

    #[snafu(display("Retrieving password using pass failed: {}", source))]
    PassEntry { source: std::io::Error },

    #[snafu(display("No pass entry given, but required"))]
    NoPassEntry,

    #[snafu(display("No password provided"))]
    NoPassword,

    #[snafu(display("No account name provided"))]
    NoAccount,

    #[snafu(display("Login failed"))]
    LoginFailed,

    #[snafu(display("Invalid password (non-unicode) in environment variable"))]
    InvalidPasswordEnv,

    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },
}

impl Cmd for Input {
    type CmdError = Error;
    fn exec(&self, ctx: &Context) -> Result<(), Error> {
        let mut result = login(self, ctx)?;
        if result.require_second_factor {
            log::info!("Account has two-factor auth enabled. Sending otp now.");
            result = login_otp(self, ctx)?;
        }

        ctx.write_result(result).context(WriteResultSnafu)?;
        Ok(())
    }
}

pub fn login(opts: &Input, ctx: &Context) -> Result<AuthResp, Error> {
    let body = AuthRequest {
        account: get_account(opts, ctx)?,
        password: get_password(opts, ctx)?,
        remember_me: false,
    };
    ctx.client.login(&body).context(HttpClientSnafu)
}

pub fn login_otp(opts: &Input, ctx: &Context) -> Result<AuthResp, Error> {
    let otp = get_otp(opts, ctx)?;
    ctx.client.login_otp(&otp).context(HttpClientSnafu)
}

/// Get the OTP code in this order:
///
/// * Check options or the config for a otp pass entry. Obtain the
///   secret and calculate the current OTP
/// * Ask the user for the OTP
fn get_otp(opts: &Input, ctx: &Context) -> Result<String, Error> {
    let totp_entry = opts
        .pass_otp
        .clone()
        .or_else(|| ctx.cfg.pass_otp_secret.clone());

    match totp_entry {
        None => {
            print!("Authentication code: ");
            std::io::stdout().flush().context(PassEntrySnafu)?;
            let mut otp: String = String::new();
            std::io::stdin()
                .read_line(&mut otp)
                .context(PassEntrySnafu)?;
            Ok(otp.trim().to_string())
        }
        Some(name) => {
            log::debug!("Looking up TOTP secret via: {}", name);
            if let Some(secret) = name.strip_prefix("key:") {
                log::debug!("Looking up a line in {:?}", ctx.cfg.pass_entry);
                let pentry = ctx.cfg.pass_entry.clone().ok_or(Error::NoPassEntry)?;
                let otp_secret = pass::pass_key(&pentry, secret).context(PassEntrySnafu)?;
                let otp = TOTP::new(otp_secret).now();
                Ok(otp.trim().to_string())
            } else {
                log::debug!("Retrieve totp secret from separate entry");
                let otp_secret = pass::pass_password(&name).context(PassEntrySnafu)?;
                let otp = TOTP::new(otp_secret).now();
                Ok(otp.trim().to_string())
            }
        }
    }
}

/// Get the password in this order:
/// * Check options for password or pass_entry
/// * Check environment variable DSC_PASSWORD
/// * Check config file
fn get_password(opts: &Input, ctx: &Context) -> Result<String, Error> {
    if let Some(pe) = &opts.pass_entry {
        log::debug!("Using given pass entry");
        pass::pass_password(pe).context(PassEntrySnafu)
    } else if let Some(pw) = &opts.password {
        log::debug!("Using given plain password");
        Ok(pw.clone())
    } else {
        match std::env::var_os(DSC_PASSWORD) {
            Some(pw) => {
                log::debug!("Using password from environment variable.");
                pw.into_string().map_err(|_os| Error::InvalidPasswordEnv)
            }
            None => match &ctx.cfg.pass_entry {
                Some(pe) => {
                    log::debug!("Using pass_entry from config file.");
                    pass::pass_password(pe).context(PassEntrySnafu)
                }
                None => Err(Error::NoPassword),
            },
        }
    }
}

fn get_account(opts: &Input, ctx: &Context) -> Result<String, Error> {
    let acc = match &opts.user {
        Some(u) => Ok(u.clone()),
        None => ctx.cfg.default_account.clone().ok_or(Error::NoAccount),
    };
    log::debug!("Using account: {:?}", &acc);
    acc
}

const DSC_PASSWORD: &str = "DSC_PASSWORD";
