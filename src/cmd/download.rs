use crate::cmd::search;
use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::file;
use crate::{cmd::login, types::DOCSPELL_AUTH};
use clap::Clap;
use reqwest::blocking::Response;
use snafu::{ResultExt, Snafu};
use std::{collections::HashMap, path::PathBuf};

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

    /// Creates a single zip file containing all files (flat). If this
    /// is enabled, the `target` option is expected to be the target
    /// zip file and not a directory.
    #[clap(long)]
    zip: bool,

    /// Download everything into this directory. If not given, the
    /// current working directory is used. If `--zip` is used, this is
    /// the zip file to create.
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

    #[snafu(display("Error creating zip file. {}", source))]
    Zip { source: zip::result::ZipError },
}

#[derive(Debug)]
struct Attach {
    name: String,
    id: String,
}
impl Attach {
    fn to_url(&self, args: &CmdArgs) -> String {
        format!("{}/api/v1/sec/attachment/{}", args.docspell_url(), self.id)
    }

    fn download(&self, args: &CmdArgs) -> Result<Response, Error> {
        let url = &self.to_url(args);
        let token = login::session_token(args).context(Login)?;
        let client = reqwest::blocking::Client::new();
        client
            .get(url)
            .header(DOCSPELL_AUTH, &token)
            .send()
            .and_then(|r| r.error_for_status())
            .context(Http { url })
    }
}

pub fn download(opts: &Input, args: &CmdArgs) -> Result<(), Error> {
    let attachs = get_attachments(opts, args)?;
    if attachs.is_empty() {
        println!("The search result is empty.");
        Ok(())
    } else {
        match opts.zip {
            true => {
                let zip_file = opts
                    .target
                    .clone()
                    .unwrap_or(PathBuf::from("docspell-files.zip"));
                if let Some(parent) = zip_file.parent() {
                    if !parent.exists() {
                        std::fs::create_dir_all(&parent).context(CreateFile)?;
                    }
                }
                println!(
                    "Downloading {} attachments into {} …",
                    attachs.len(),
                    zip_file.display()
                );

                download_zip(attachs, opts, args, &zip_file)
            }
            false => {
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

                download_flat(attachs, opts, args, &parent)
            }
        }
    }
}

fn download_flat(
    attachs: Vec<Attach>,
    opts: &Input,
    args: &CmdArgs,
    parent: &PathBuf,
) -> Result<(), Error> {
    let mut dupes = Dupes::new();
    for s in attachs {
        let mut resp = s.download(args)?;
        let fname = dupes.use_name(get_filename(&resp).unwrap_or(&s.name));
        let path = parent.join(&fname);
        if path.exists() && !opts.overwrite {
            println!("File exists: {}. Skipping.", path.display());
        } else {
            println!("Downloading {} …", &fname);
            let file = std::fs::File::create(path).context(CreateFile)?;
            let mut writer = std::io::BufWriter::new(file);
            resp.copy_to(&mut writer).context(ReadResponse)?;
        }
    }
    Ok(())
}

fn download_zip(
    attachs: Vec<Attach>,
    opts: &Input,
    args: &CmdArgs,
    zip_file: &PathBuf,
) -> Result<(), Error> {
    if zip_file.exists() && !opts.overwrite {
        println!("Zip file already exists! {}", zip_file.display());
    } else {
        if zip_file.exists() {
            std::fs::remove_file(zip_file).context(CreateFile)?;
        }
        let zip = std::fs::File::create(zip_file).context(CreateFile)?;
        let mut zw = zip::ZipWriter::new(zip);
        let mut dupes = Dupes::new();
        for s in attachs {
            let mut resp = s.download(args)?;
            let fname = dupes.use_name(get_filename(&resp).unwrap_or(&s.name));
            zw.start_file(&fname, zip::write::FileOptions::default())
                .context(Zip)?;
            println!("Downloading {} …", &fname);
            resp.copy_to(&mut zw).context(ReadResponse)?;
        }

        zw.finish().context(Zip)?;
    }
    Ok(())
}

fn get_filename<'a>(resp: &'a Response) -> Option<&'a str> {
    resp.headers()
        .get("Content-Disposition")
        .and_then(|hv| hv.to_str().ok())
        .and_then(file::filename_from_header)
}

fn get_attachments(opts: &Input, args: &CmdArgs) -> Result<Vec<Attach>, Error> {
    let search_params = search::Input {
        query: opts.query.clone(),
        offset: 0,
        limit: 20,
        with_details: true,
    };
    let result = search::search(&search_params, args).context(Search)?;
    let mut attachs: Vec<Attach> = Vec::new();
    for g in result.groups {
        for item in g.items {
            for a in item.attachments {
                attachs.push(Attach {
                    id: a.id.clone(),
                    name: a.name.unwrap_or(format!("{}.pdf", a.id)),
                });
            }
        }
    }

    Ok(attachs)
}

struct Dupes {
    names: HashMap<String, i32>,
}

impl Dupes {
    fn new() -> Dupes {
        Dupes {
            names: HashMap::new(),
        }
    }

    fn use_name(&mut self, name: &str) -> String {
        let fname = name.to_string();
        match self.names.get(&fname) {
            Some(count) => {
                let next_name = file::splice_name(name, count);
                let next_count = count + 1;
                self.names.insert(fname.clone(), next_count);
                next_name
            }
            None => {
                self.names.insert(fname.clone(), 1);
                fname
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn unit_dupes_add() {
        let mut dupes = Dupes::new();
        assert_eq!(dupes.use_name("test.png"), "test.png");
        assert_eq!(dupes.use_name("test.png"), "test_1.png");
        assert_eq!(dupes.use_name("test.png"), "test_2.png");
    }
}
