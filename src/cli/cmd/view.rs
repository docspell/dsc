use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::{cmd::login, types::DOCSPELL_AUTH};
use crate::{cmd::search, types::Attach};
use clap::{ArgGroup, Clap};
use dialoguer::Confirm;
use reqwest::blocking::Response;
use snafu::{ResultExt, Snafu};
use std::{path::PathBuf, process::Command};

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

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        view(self, args).map_err(|source| CmdError::View { source })?;
        Ok(())
    }
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

    #[snafu(display("Error while searching. {}", source))]
    Search { source: search::Error },

    #[snafu(display("Error creating a file. {}", source))]
    CreateFile { source: std::io::Error },

    #[snafu(display("Error executing command: {}", source))]
    Exec { source: std::io::Error },

    #[snafu(display("No pdf viewer defined in the config file!"))]
    NoPdfViewer,

    #[snafu(display("Interaction with terminal failed: {}", source))]
    Interact { source: std::io::Error },
}

pub fn view(opts: &Input, args: &CmdArgs) -> Result<(), Error> {
    let parent = std::env::temp_dir().join("dsc-view");

    if !parent.exists() {
        std::fs::create_dir_all(&parent).context(CreateFile)?;
    }

    view_all(opts, args, &parent)
}

pub fn view_all(opts: &Input, args: &CmdArgs, parent: &PathBuf) -> Result<(), Error> {
    let search_params = search::Input {
        query: opts.query.clone(),
        offset: opts.offset,
        limit: opts.limit,
        with_details: true,
    };
    let result = search::search(&search_params, args).context(Search)?;

    let mut confirm = false;
    for g in result.groups {
        for item in g.items {
            for a in item.attachments {
                if confirm {
                    if is_stop_viewing(opts)? {
                        return Ok(());
                    }
                } else {
                    confirm = true;
                }

                let file = download(a, args, parent)?;
                let tool = &args.cfg.pdf_viewer.get(0).ok_or(Error::NoPdfViewer)?;
                let tool_args: Vec<String> = args
                    .cfg
                    .pdf_viewer
                    .iter()
                    .skip(1)
                    .map(|s| s.replace("{}", file.display().to_string().as_str()))
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
            }
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

fn download(attach: Attach, args: &CmdArgs, parent: &PathBuf) -> Result<PathBuf, Error> {
    let mut resp = attach.download(args)?;
    let path = parent.join("view.pdf");

    if path.exists() {
        std::fs::remove_file(&path).context(CreateFile)?;
    }

    let file = std::fs::File::create(&path).context(CreateFile)?;
    let mut writer = std::io::BufWriter::new(file);
    resp.copy_to(&mut writer).context(ReadResponse)?;

    Ok(path)
}

impl Attach {
    fn to_url(&self, args: &CmdArgs) -> String {
        format!("{}/api/v1/sec/attachment/{}", args.docspell_url(), self.id)
    }

    fn download(&self, args: &CmdArgs) -> Result<Response, Error> {
        let url = self.to_url(args);
        let token = login::session_token(args).context(Login)?;
        args.client
            .get(&url)
            .header(DOCSPELL_AUTH, &token)
            .send()
            .and_then(|r| r.error_for_status())
            .context(Http { url: url.clone() })
    }
}
