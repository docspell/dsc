use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::types::{GenInvite, InviteResult};
use clap::Clap;
use snafu::{ResultExt, Snafu};

/// Generates a new invitation key.
///
/// The password can be found in the config file of the Docspell
/// server.
#[derive(Clap, Debug)]
pub struct Input {
    #[clap(long, short)]
    password: String,
}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        let result = gen_invite(self, args).map_err(|source| CmdError::GenInvite { source })?;
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

pub fn gen_invite(opts: &Input, args: &CmdArgs) -> Result<InviteResult, Error> {
    let url = &format!("{}/api/v1/open/signup/newinvite", args.docspell_url());
    args.client
        .post(url)
        .json(&GenInvite {
            password: opts.password.clone(),
        })
        .send()
        .context(Http { url })?
        .json::<InviteResult>()
        .context(ReadResponse)
}
