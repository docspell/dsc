use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::opts::ConfigOpts;
use crate::types::{BasicResult, Registration};
use clap::Clap;

/// Register a new account at Docspell.
#[derive(Clap, Debug)]
pub struct Input {
    /// The collective name to use. If unsure, use the same as login.
    #[clap(long, short)]
    pub collective_name: String,

    /// The user name. This name together with the collective name
    /// must be unique.
    #[clap(long, short)]
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
        let result = gen_invite(self, args.opts).and_then(|r| args.make_str(&r));
        println!("{:}", result?);
        Ok(())
    }
}

fn gen_invite(args: &Input, cfg: &ConfigOpts) -> Result<BasicResult, CmdError> {
    let url = format!("{}/api/v1/open/signup/register", cfg.docspell_url);
    let body = &Registration {
        collective_name: args.collective_name.clone(),
        login: args.login.clone(),
        password: args.password.clone(),
        invite: args.invite.clone(),
    };
    log::debug!("Register new account: {:?}", body);
    let client = reqwest::blocking::Client::new();
    return client
        .post(url)
        .json(body)
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(CmdError::HttpError)?
        .json::<BasicResult>()
        .map_err(CmdError::HttpError);
}
