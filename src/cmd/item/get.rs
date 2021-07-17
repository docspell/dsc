use crate::types::DOCSPELL_AUTH;
use crate::{cmd::login, types::ItemDetail};
use crate::{
    cmd::{search, Cmd, CmdArgs, CmdError},
    types::SearchResult,
};
use clap::Clap;
use reqwest::StatusCode;
use snafu::{ResultExt, Snafu};

/// List all sources for your collective
#[derive(Clap, Debug)]
pub struct Input {
    /// The item id (can be abbreviated)
    pub id: String,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error received from server at {}: {}", url, source))]
    Http { source: reqwest::Error, url: String },

    #[snafu(display("Error received from server: {}", source))]
    ReadResponse { source: reqwest::Error },

    #[snafu(display(
        "Error logging in via session. Consider the `login` command. {}",
        source
    ))]
    Login { source: login::Error },

    #[snafu(display("Could not query the complete id: {}", source))]
    IdSearch { source: search::Error },

    #[snafu(display("The item was not found"))]
    ItemNotFound,

    #[snafu(display("The partial id resulted in multiple items."))]
    ItemNotUnique,
}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        let item =
            get_item(self.id.as_str(), args).map_err(|source| CmdError::ItemGet { source })?;
        args.write_result(item)?;
        Ok(())
    }
}

pub fn get_item(id: &str, args: &CmdArgs) -> Result<ItemDetail, Error> {
    let item_id = if id.len() < 47 {
        get_item_id(id, args)?
    } else {
        id.into()
    };

    let url = &format!("{}/api/v1/sec/item/{}", args.docspell_url(), item_id);
    let token = login::session_token(args).context(Login)?;
    let resp = args
        .client
        .get(url)
        .header(DOCSPELL_AUTH, token)
        .send()
        .context(Http { url })?;

    if resp.status() == StatusCode::NOT_FOUND {
        Err(Error::ItemNotFound)
    } else {
        resp.error_for_status()
            .context(Http { url })?
            .json::<ItemDetail>()
            .context(ReadResponse)
    }
}

fn get_item_id(partial_id: &str, args: &CmdArgs) -> Result<String, Error> {
    log::debug!(
        "Item id '{}' is not complete, searching for the item via a query",
        partial_id
    );
    search::search(
        &search::Input {
            query: format!("id:{}*", partial_id),
            offset: 0,
            limit: 2,
            with_details: false,
        },
        args,
    )
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
