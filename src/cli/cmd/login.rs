use super::{Cmd, Context};
use crate::cli::sink::Error as SinkError;
use crate::http::payload::{AuthRequest, AuthResp};
use crate::http::Error as HttpError;

use crate::util::pass;

use clap::{ArgGroup, Parser, ValueHint};
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
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An http error occurred: {}!", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error received from server: {}", source))]
    ReadResponse { source: reqwest::Error },

    #[snafu(display("Retrieving password using pass failed: {}", source))]
    PassEntry { source: std::io::Error },

    #[snafu(display("No password provided!"))]
    NoPassword,

    #[snafu(display("No account name provided!"))]
    NoAccount,

    #[snafu(display("Login failed!"))]
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
            result = login_otp(ctx)?;
        }

        ctx.write_result(result).context(WriteResult)?;
        Ok(())
    }
}

pub fn login(opts: &Input, ctx: &Context) -> Result<AuthResp, Error> {
    let body = AuthRequest {
        account: get_account(opts, ctx)?,
        password: get_password(opts, ctx)?,
        remember_me: false,
    };
    ctx.client.login(&body).context(HttpClient)
}

pub fn login_otp(ctx: &Context) -> Result<AuthResp, Error> {
    print!("Authentication code: ");
    std::io::stdout().flush().context(PassEntry)?;
    let mut otp: String = String::new();
    std::io::stdin().read_line(&mut otp).context(PassEntry)?;
    ctx.client.login_otp(otp.trim()).context(HttpClient)
}

fn get_password(opts: &Input, ctx: &Context) -> Result<String, Error> {
    match ctx.pass_entry(&opts.pass_entry) {
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

fn get_account(opts: &Input, ctx: &Context) -> Result<String, Error> {
    let acc = match &opts.user {
        Some(u) => Ok(u.clone()),
        None => ctx.cfg.default_account.clone().ok_or(Error::NoAccount),
    };
    log::debug!("Using account: {:?}", &acc);
    acc
}

const DSC_PASSWORD: &str = "DSC_PASSWORD";
