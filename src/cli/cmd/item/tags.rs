use clap::{ArgGroup, Parser};
use snafu::{ResultExt, Snafu};

use super::{Cmd, Context};
use crate::cli::sink::Error as SinkError;
use crate::http::payload::{BasicResult, StringList};
use crate::http::Error as HttpError;

/// Add or remove tags for an item.
#[derive(Parser, Debug)]
#[clap(group = ArgGroup::new("action"))]
pub struct Input {
    /// The item id (can be abbreviated to a prefix)
    #[clap(long)]
    pub id: String,

    /// Add the given tags.
    #[clap(long, group = "action")]
    pub add: bool,

    /// Remove the given tags.
    #[clap(long, group = "action")]
    pub remove: bool,

    /// Replace all item tags with the given ones.
    #[clap(long, group = "action")]
    pub replace: bool,

    /// A list of tags. Can be ids or names.
    #[clap(required = true, min_values = 1)]
    pub tags: Vec<String>,
}

impl Input {
    fn to_action(&self) -> Result<Action, Error> {
        if self.remove {
            Ok(Action::Remove)
        } else if self.replace {
            Ok(Action::Replace)
        } else if self.add {
            Ok(Action::Add)
        } else {
            Err(Error::NoAction)
        }
    }
}

enum Action {
    Add,
    Remove,
    Replace,
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
            Action::Add => add_tags(self, ctx)?,
            Action::Replace => replace_tags(self, ctx)?,
            Action::Remove => remove_tags(self, ctx)?,
        };
        ctx.write_result(result).context(WriteResult)?;
        Ok(())
    }
}

fn add_tags(opts: &Input, ctx: &Context) -> Result<BasicResult, Error> {
    let tags = StringList {
        items: opts.tags.clone(),
    };
    ctx.client
        .link_tags(&ctx.opts.session, &opts.id, &tags)
        .context(HttpClient)
}

fn replace_tags(opts: &Input, ctx: &Context) -> Result<BasicResult, Error> {
    let tags = StringList {
        items: opts.tags.clone(),
    };
    ctx.client
        .set_tags(&ctx.opts.session, &opts.id, &tags)
        .context(HttpClient)
}

fn remove_tags(opts: &Input, ctx: &Context) -> Result<BasicResult, Error> {
    let tags = StringList {
        items: opts.tags.clone(),
    };
    ctx.client
        .remove_tags(&ctx.opts.session, &opts.id, &tags)
        .context(HttpClient)
}
