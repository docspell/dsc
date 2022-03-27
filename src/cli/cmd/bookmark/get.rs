use clap::Parser;
use snafu::{ResultExt, Snafu};

use super::{Cmd, Context};
use crate::cli::sink::Error as SinkError;
use crate::http::payload::{Bookmark, BookmarkList};
use crate::http::Error as HttpError;

/// Gets all bookmarks.
#[derive(Parser, Debug)]
pub struct Input {}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An http error occurred: {}", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },

    #[snafu(display("The item was not found"))]
    ItemNotFound,
}

impl Cmd for Input {
    type CmdError = Error;

    fn exec(&self, ctx: &Context) -> Result<(), Error> {
        let bookmarks = get_bookmarks(ctx)?;
        let bmlist = BookmarkList { bookmarks };
        ctx.write_result(bmlist).context(WriteResultSnafu)?;
        Ok(())
    }
}

fn get_bookmarks(ctx: &Context) -> Result<Vec<Bookmark>, Error> {
    ctx.client
        .get_bookmarks(&ctx.opts.session)
        .context(HttpClientSnafu)
}
