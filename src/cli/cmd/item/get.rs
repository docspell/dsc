use clap::Clap;
use snafu::{ResultExt, Snafu};

use super::{Cmd, Context};
use crate::cli::sink::Error as SinkError;
use crate::http::payload::ItemDetail;
use crate::http::Error as HttpError;

/// List all sources for your collective
#[derive(Clap, Debug)]
pub struct Input {
    /// The item id (can be abbreviated)
    pub id: String,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An http error occurred: {}!", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },

    #[snafu(display("Could not query the complete id: {}", source))]
    IdSearch { source: HttpError },

    #[snafu(display("The item was not found"))]
    ItemNotFound,
}

impl Cmd for Input {
    type CmdError = Error;

    fn exec(&self, ctx: &Context) -> Result<(), Error> {
        let item = get_item(self.id.as_str(), ctx)?;
        ctx.write_result(item).context(WriteResult)?;
        Ok(())
    }
}

fn get_item(id: &str, ctx: &Context) -> Result<ItemDetail, Error> {
    let result = ctx
        .client
        .get_item(&ctx.opts.session, id)
        .context(HttpClient)?;

    result.ok_or(Error::ItemNotFound)
}
