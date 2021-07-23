use clap::{ArgGroup, Clap};
use dialoguer::Confirm;
use snafu::{ResultExt, Snafu};
use std::{path::PathBuf, process::Command};

use super::{Cmd, Context};
use crate::http::payload::SearchReq;
use crate::http::DownloadRef;
use crate::http::Error as HttpError;

/// View pdf files.
///
/// Searches for documents via a query and downloads one at a time to
/// feed it into a viewer program. The prorgam can be defined in the
/// config file.
///
/// Use the `search-summary` command with the same query to get an
/// idea how much is being downloaded. This is an interactive command.
#[derive(Clap, std::fmt::Debug)]
#[clap(group = ArgGroup::new("kind"))]
pub struct Input {
    /// The query string. See https://docspell.org/docs/query/
    query: String,

    /// Limit the number of results.
    #[clap(short, long, default_value = "60")]
    limit: u32,

    /// Skip the first n results.
    #[clap(short, long, default_value = "0")]
    offset: u32,

    /// Ask whether to keep viewing between each file.
    #[clap(long, short)]
    stop: bool,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An http error occurred: {}!", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error creating a file. {}", source))]
    CreateFile { source: std::io::Error },

    #[snafu(display("Error executing command: {}", source))]
    Exec { source: std::io::Error },

    #[snafu(display("No pdf viewer defined in the config file!"))]
    NoPdfViewer,

    #[snafu(display("Interaction with terminal failed: {}", source))]
    Interact { source: std::io::Error },
}

impl Cmd for Input {
    type CmdError = Error;

    fn exec(&self, ctx: &Context) -> Result<(), Error> {
        let parent = std::env::temp_dir().join("dsc-view");

        if !parent.exists() {
            std::fs::create_dir_all(&parent).context(CreateFile)?;
        }

        view_all(self, ctx, &parent)
    }
}

pub fn view_all(opts: &Input, ctx: &Context, parent: &PathBuf) -> Result<(), Error> {
    let req = SearchReq {
        query: opts.query.clone(),
        offset: opts.offset,
        limit: opts.limit,
        with_details: true,
    };
    let result = ctx
        .client
        .download_search(&ctx.opts.session, &req)
        .context(HttpClient)?;

    let mut confirm = false;
    for dref in result {
        if confirm {
            if is_stop_viewing(opts)? {
                return Ok(());
            }
        } else {
            confirm = true;
        }

        let file = download(&dref, ctx, parent)?;
        if let Some(f) = file {
            let tool = &ctx.cfg.pdf_viewer.get(0).ok_or(Error::NoPdfViewer)?;
            let tool_args: Vec<String> = ctx
                .cfg
                .pdf_viewer
                .iter()
                .skip(1)
                .map(|s| s.replace("{}", f.display().to_string().as_str()))
                .collect();
            log::info!(
                "Run: {} {}",
                tool,
                tool_args
                    .iter()
                    .map(|s| format!("'{}'", s))
                    .collect::<Vec<String>>()
                    .join(" ")
            );
            Command::new(tool).args(tool_args).output().context(Exec)?;
        } else {
            eprintln!(
                "Skip attachment: {}/{}. There was no file!",
                dref.id, dref.name
            );
        }
    }

    Ok(())
}

fn is_stop_viewing(opts: &Input) -> Result<bool, Error> {
    if opts.stop {
        if let Some(answer) = Confirm::new()
            .with_prompt("Keep viewing?")
            .interact_opt()
            .context(Interact)?
        {
            return Ok(!answer);
        }
    } else {
        return Ok(false);
    }
    return Ok(false);
}

fn download(
    attach: &DownloadRef,
    ctx: &Context,
    parent: &PathBuf,
) -> Result<Option<PathBuf>, Error> {
    let dlopt = attach
        .get(&ctx.client, &ctx.opts.session)
        .context(HttpClient)?;

    let path = parent.join("view.pdf");

    if let Some(mut dl) = dlopt {
        if path.exists() {
            std::fs::remove_file(&path).context(CreateFile)?;
        }

        let file = std::fs::File::create(&path).context(CreateFile)?;
        let mut writer = std::io::BufWriter::new(file);
        dl.copy_to(&mut writer).context(HttpClient)?;
        Ok(Some(path))
    } else {
        Ok(None)
    }
}
