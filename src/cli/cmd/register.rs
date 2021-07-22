use clap::{Clap, ValueHint};
use snafu::{ResultExt, Snafu};

use super::{Cmd, Context};
use crate::cli::sink::Error as SinkError;
use crate::http::payload::Registration;
use crate::http::Error as HttpError;

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
    type CmdError = Error;

    fn exec(&self, ctx: &Context) -> Result<(), Error> {
        let body = Registration {
            collective_name: self.collective_name.clone(),
            login: self.login.clone(),
            password: self.password.clone(),
            invite: self.invite.clone(),
        };

        let result = ctx.client.register(&body).context(HttpClient)?;
        ctx.write_result(result).context(WriteResult)?;
        Ok(())
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An http error occurred: {}!", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },
}
