use clap::{ArgGroup, Clap};
use snafu::{ResultExt, Snafu};

use super::{Cmd, Context};
use crate::cli::sink::Error as SinkError;
use crate::http::payload::{BasicResult, CustomFieldValue};
use crate::http::Error as HttpError;

/// Set or remove field values for an item.
#[derive(Clap, Debug)]
#[clap(group = ArgGroup::new("action"))]
pub struct Input {
    /// The item id (can be abbreviated to a prefix)
    #[clap(long)]
    pub id: String,

    /// Set the value of the field.
    #[clap(long, group = "action")]
    pub set: Option<String>,

    /// Remove the field from the item.
    #[clap(long, group = "action")]
    pub remove: bool,

    /// The field name.
    #[clap(long)]
    pub name: String,
}

impl Input {
    fn to_action(&self) -> Result<Action, Error> {
        if self.remove {
            Ok(Action::Remove)
        } else {
            match &self.set {
                Some(v) => Ok(Action::Set(v.clone())),
                None => Err(Error::NoAction),
            }
        }
    }
}

enum Action {
    Set(String),
    Remove,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An http error occurred: {}!", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },

    #[snafu(display("No action given!"))]
    NoAction,
}

impl Cmd for Input {
    type CmdError = Error;

    fn exec(&self, ctx: &Context) -> Result<(), Error> {
        let result = match self.to_action()? {
            Action::Set(value) => set_field(&self.name, value, &self.id, ctx)?,
            Action::Remove => remove_field(self, ctx)?,
        };
        ctx.write_result(result).context(WriteResult)?;
        Ok(())
    }
}

fn set_field(
    name: &String,
    value: String,
    id: &String,
    ctx: &Context,
) -> Result<BasicResult, Error> {
    let fvalue = CustomFieldValue {
        field: name.clone(),
        value,
    };
    ctx.client
        .set_field(&ctx.opts.session, id, &fvalue)
        .context(HttpClient)
}

fn remove_field(opts: &Input, ctx: &Context) -> Result<BasicResult, Error> {
    ctx.client
        .remove_field(&ctx.opts.session, &opts.id, &opts.name)
        .context(HttpClient)
}
