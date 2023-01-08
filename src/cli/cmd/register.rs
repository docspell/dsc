use clap::{Parser, ValueHint};
use snafu::{ResultExt, Snafu};

use super::{Cmd, Context};
use crate::cli::sink::Error as SinkError;
use crate::http::payload::Registration;
use crate::http::Error as HttpError;

/// Register a new account at Docspell.
#[derive(Parser, Debug)]
pub struct Input {
    /// The collective name to use. If unsure, use the same as login.
    #[arg(long, short, value_hint = ValueHint::Username)]
    pub collective_name: String,

    /// The user name. This name together with the collective name
    /// must be unique.
    #[arg(long, short, value_hint = ValueHint::Username)]
    pub login: String,

    /// The password for the account.
    #[arg(long, short)]
    pub password: String,

    /// If signup requires an invitation key, it can be specified
    /// here.
    #[arg(long, short)]
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

        let result = ctx.client.register(&body).context(HttpClientSnafu)?;
        ctx.write_result(result).context(WriteResultSnafu)?;
        Ok(())
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An http error occurred: {}", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },
}
