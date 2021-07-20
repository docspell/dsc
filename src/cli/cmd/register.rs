use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::types::{BasicResult, Registration};
use clap::{Clap, ValueHint};
use snafu::{ResultExt, Snafu};

/// Register a new account at Docspell.
#[derive(Clap, Debug)]
pub struct Input {
    /// The collective name to use. If unsure, use the same as login.
    #[clap(long, short, value_hint = ValueHint::Username)]
    pub collective_name: String,

    /// The user name. This name together with the collective name
    /// must be unique.
    #[clap(long, short, value_hint = ValueHint::Username)]
    pub login: String,

    /// The password for the account.
    #[clap(long, short)]
    pub password: String,

    /// If signup requires an invitation key, it can be specified
    /// here.
    #[clap(long, short)]
    pub invite: Option<String>,
}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        let result = register(self, args).map_err(|source| CmdError::Register { source })?;
        args.write_result(result)?;
        Ok(())
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error received from server at {}: {}", url, source))]
    Http { source: reqwest::Error, url: String },

    #[snafu(display("Error received from server: {}", source))]
    ReadResponse { source: reqwest::Error },
}

fn register(opts: &Input, args: &CmdArgs) -> Result<BasicResult, Error> {
    let url = &format!("{}/api/v1/open/signup/register", args.docspell_url());
    let body = &Registration {
        collective_name: opts.collective_name.clone(),
        login: opts.login.clone(),
        password: opts.password.clone(),
        invite: opts.invite.clone(),
    };
    log::debug!("Register new account: {:?}", body);
    args.client
        .post(url)
        .json(body)
        .send()
        .and_then(|r| r.error_for_status())
        .context(Http { url })?
        .json::<BasicResult>()
        .context(ReadResponse)
}
