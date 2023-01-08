use clap::Parser;
use prettytable::{row, Table};
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use std::path::{Path, PathBuf};
use webbrowser;

use crate::cli::cmd;
use crate::cli::opts::EndpointOpts;
use crate::cli::sink::{Error as SinkError, Sink};
use crate::cli::table;
use crate::http::payload::CheckFileResult;
use crate::http::Error as HttpError;
use crate::util::digest;

use super::{Cmd, Context};

/// Open the item to a file, or given via the item id with the default
/// browser.
#[derive(Parser, std::fmt::Debug)]
pub struct Input {
    #[clap(flatten)]
    pub endpoint: EndpointOpts,

    /// Do not open the item in the browser, only print the url.
    #[arg(long)]
    pub print_only: bool,

    /// A file or an item id.
    pub file_or_item: String,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An http error occurred: {}", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Collective must be present when using integration endpoint."))]
    NoCollective,

    #[snafu(display("Calculating digest of file {} failed: {}", path.display(), source))]
    DigestFail {
        source: std::io::Error,
        path: PathBuf,
    },

    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },

    #[snafu(display("Error opening browser: {}", source))]
    Webbrowser { source: std::io::Error },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CmdResult {
    pub success: bool,
    pub has_more: bool,
    pub item_id: Option<String>,
    pub url: Option<String>,
}
impl table::AsTable for CmdResult {
    fn to_table(&self) -> Table {
        let mut table = table::mk_table();
        table.set_titles(row![bFg => "success", "item id", "url"]);
        table.add_row(row![
            self.success,
            self.item_id.clone().unwrap_or_else(|| String::from("-")),
            self.url.clone().unwrap_or_else(|| String::from("-")),
        ]);
        table
    }
}
impl Sink for CmdResult {}

impl CmdResult {
    pub fn new(item_id: String, more: bool, ctx: &Context) -> CmdResult {
        CmdResult {
            success: true,
            has_more: more,
            url: Some(create_url(ctx, &item_id)),
            item_id: Some(item_id),
        }
    }
    pub fn none() -> CmdResult {
        CmdResult {
            success: false,
            has_more: false,
            item_id: None,
            url: None,
        }
    }
}

impl Cmd for Input {
    type CmdError = Error;

    fn exec(&self, ctx: &Context) -> Result<(), Error> {
        let path = Path::new(&self.file_or_item);
        let result = if path.is_file() {
            let result = item_from_file(path, &self.endpoint, ctx)?;
            let mut iter = result.items.into_iter();
            match iter.next() {
                None => {
                    log::info!("No items found for file {}", path.display());
                    CmdResult::none()
                }
                Some(item) => {
                    let more = iter.next().is_some();
                    if more {
                        log::info!(
                            "More than one item for file {}. Using first.",
                            path.display()
                        );
                    }
                    CmdResult::new(item.id, more, ctx)
                }
            }
        } else {
            // interpret it as id
            CmdResult::new(self.file_or_item.clone(), false, ctx)
        };

        let the_url = &result.url.clone();
        ctx.write_result(result).context(WriteResultSnafu)?;

        if let Some(url) = &the_url {
            if !self.print_only {
                webbrowser::open(url).context(WebbrowserSnafu)?;
            }
        }
        Ok(())
    }
}

fn create_url(ctx: &Context, item_id: &str) -> String {
    let base_url = cmd::docspell_url(ctx.opts, ctx.cfg);
    format!("{}/app/item/{}", base_url, item_id)
}

fn item_from_file(
    file: &Path,
    opts: &EndpointOpts,
    ctx: &Context,
) -> Result<CheckFileResult, Error> {
    let fa = opts
        .to_file_auth(ctx, &|| None)
        .ok_or(Error::NoCollective)?;
    let hash = digest::digest_file_sha256(file).context(DigestFailSnafu { path: file })?;
    let mut result = ctx.client.file_exists(hash, &fa).context(HttpClientSnafu)?;
    result.file = file.canonicalize().ok().map(|p| p.display().to_string());
    Ok(result)
}
