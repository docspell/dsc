use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::opts::ConfigOpts;
use crate::types::{GenInvite, InviteResult};
use clap::Clap;

/// Generates a new invitation key.
#[derive(Clap, Debug)]
pub struct Input {
    #[clap(long, short)]
    password: String,
}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        let result = gen_invite(self, args.opts).and_then(|r| args.make_str(&r));
        println!("{:}", result?);
        Ok(())
    }
}

fn gen_invite(args: &Input, cfg: &ConfigOpts) -> Result<InviteResult, CmdError> {
    let url = format!("{}/api/v1/open/signup/newinvite", cfg.docspell_url);
    let client = reqwest::blocking::Client::new();
    return client
        .post(url)
        .json(&GenInvite {
            password: args.password.clone(),
        })
        .send()
        .map_err(CmdError::HttpError)?
        .json::<InviteResult>()
        .map_err(CmdError::HttpError);
}
