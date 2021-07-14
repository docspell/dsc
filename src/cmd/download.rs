use crate::cmd::search;
use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::{cmd::login, types::DOCSPELL_AUTH};
use clap::Clap;
use snafu::{ResultExt, Snafu};
use std::{io::Cursor, path::PathBuf};

/// Searches for documents via a query and downloads all associated
/// files.
///
/// Use `search-summary` with the same query to get an idea how much
/// is being downloaded.
#[derive(Clap, std::fmt::Debug)]
pub struct Input {
    /// The query string. See https://docspell.org/docs/query/
    query: String,

    /// Limit the number of results.
    #[clap(short, long, default_value = "20")]
    limit: u32,

    /// Skip the first n results.
    #[clap(short, long, default_value = "0")]
    offset: u32,

    /// Whether to overwrite existing files. They are skipped by
    /// default.
    #[clap(long)]
    overwrite: bool,

    /// Download everything into this directory. If not given, the
    /// current working directory is used.
    #[clap(short, long)]
    target: Option<PathBuf>,
}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        download(self, args).map_err(|source| CmdError::Download { source })?;
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
}

#[derive(Debug)]
struct Attach {
    name: String,
    id: String,
}

pub fn download(opts: &Input, args: &CmdArgs) -> Result<(), Error> {
    let attachs = get_attachments(opts, args)?;
    if attachs.is_empty() {
        println!("The search result is empty.");
        Ok(())
    } else {
        let parent = opts
            .target
            .clone()
            .unwrap_or(std::env::current_dir().context(CreateFile)?);

        if !parent.exists() {
            std::fs::create_dir_all(&parent).context(CreateFile)?;
        }
        println!(
            "Downloading {} attachments into {} …",
            attachs.len(),
            parent.display()
        );
        let token = login::session_token(args).context(Login)?;
        for s in attachs {
            let url = &format!("{}/api/v1/sec/attachment/{}", args.docspell_url(), s.id);

            let client = reqwest::blocking::Client::new();
            let resp = client
                .get(url)
                .header(DOCSPELL_AUTH, &token)
                .send()
                .and_then(|r| r.error_for_status())
                .context(Http { url })?;

            let mut path = parent.clone();
            path.push(&s.name);
            if path.exists() && !opts.overwrite {
                println!("File exists: {}. Skipping.", path.display());
            } else {
                let mut file = std::fs::File::create(path).context(CreateFile)?;
                let mut content = Cursor::new(resp.bytes().context(ReadResponse)?);
                println!("Downloading {} …", &s.name);
                std::io::copy(&mut content, &mut file).context(CreateFile)?;
                log::debug!("Refresh session token after download");
                login::session(&token, args).context(Login)?;
            }
        }
        Ok(())
    }
}

fn get_attachments(opts: &Input, args: &CmdArgs) -> Result<Vec<Attach>, Error> {
    let search_params = search::Input {
        query: opts.query.clone(),
        offset: 0,
        limit: 20,
        with_details: true,
    };
    let result = search::search(&search_params, args).context(Search)?;
    let x: Vec<Attach> = result
        .groups
        .iter()
        .flat_map(|g| g.items.iter())
        .map(|i| {
            i.attachments.iter().map(|a| Attach {
                id: a.id.clone(),
                name: a.name.clone().unwrap_or(format!("{}.pdf", a.id)),
            })
        })
        .flatten()
        .collect::<Vec<_>>();

    Ok(x)
}
