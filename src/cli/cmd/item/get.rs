use clap::Clap;
use snafu::{ResultExt, Snafu};

use super::{Cmd, Context};
use crate::cli::sink::Error as SinkError;
use crate::http::payload::{ItemDetail, SearchReq, SearchResult};
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

    #[snafu(display("The partial id resulted in multiple items."))]
    ItemNotUnique,
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
    let item_id = if id.len() < 47 {
        get_item_id(id, ctx)?
    } else {
        id.into()
    };

    let result = ctx
        .client
        .get_item(&ctx.opts.session, item_id)
        .context(HttpClient)?;

    result.ok_or(Error::ItemNotFound)
}

fn get_item_id(partial_id: &str, ctx: &Context) -> Result<String, Error> {
    log::debug!(
        "Item id '{}' is not complete, searching for the item via a query",
        partial_id
    );
    let req = SearchReq {
        offset: 0,
        limit: 2,
        with_details: false,
        query: format!("id:{}*", partial_id).into(),
    };
    ctx.client
        .search(&ctx.opts.session, &req)
        .context(IdSearch)
        .and_then(|r| find_id(&r))
}

fn find_id(results: &SearchResult) -> Result<String, Error> {
    match results.groups.len() {
        0 => Err(Error::ItemNotFound),
        1 => match results.groups[0].items.len() {
            0 => Err(Error::ItemNotFound),
            1 => Ok(results.groups[0].items[0].id.clone()),
            _ => Err(Error::ItemNotUnique),
        },
        _ => Err(Error::ItemNotUnique),
    }
}
