use crate::cmd::search;
use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::file;
use crate::{cmd::login, types::DOCSPELL_AUTH};
use clap::{ArgGroup, Clap};
use reqwest::{blocking::Response, StatusCode};
use snafu::{ResultExt, Snafu};
use std::{
    collections::HashMap,
    path::{Display, PathBuf},
    str::FromStr,
};

/// Searches for documents via a query and downloads all associated
/// files. It downloads by default the converted PDF files, which can
/// be changed using options `--original` and `--archive`,
/// respectively.
///
/// Use the `search-summary` command with the same query to get an
/// idea how much is being downloaded.
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

    /// Whether to overwrite already existing files. By default the
    /// download is skipped if there is already a file with the target
    /// name present. When using `--zip` this will remove an existing
    /// zip file before downloading.
    #[clap(long)]
    overwrite: bool,

    /// Download the original file instead of the converted PDF.
    #[clap(long, group = "kind")]
    original: bool,

    /// Download the original archive file to the attachment if
    /// available. Since often multiple files map to a single archive,
    /// the option `--dupes skip` can be used here.
    #[clap(long, group = "kind")]
    archive: bool,

    /// Creates a single zip file containing all files (flat). If this
    /// is enabled, the `target` option is expected to be the target
    /// zip file and not a directory.
    #[clap(long)]
    zip: bool,

    /// What to do when multiple files map to the same name. Can be
    /// one of: skip, rename. For rename, the target file is renamed
    /// by appending a number suffix.
    #[clap(long, default_value = "rename")]
    dupes: DupeMode,

    /// Download everything into this directory. If not given, the
    /// current working directory is used. If `--zip` is used, this is
    /// the zip file to create.
    #[clap(short, long)]
    target: Option<PathBuf>,
}

#[derive(Debug, Clap, PartialEq, Eq)]
pub enum DupeMode {
    Skip,
    Rename,
}
impl FromStr for DupeMode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ls = s.to_lowercase();
        if ls == "skip" {
            Ok(DupeMode::Skip)
        } else if ls == "rename" {
            Ok(DupeMode::Rename)
        } else {
            Err(Error::InvalidDupeMode { given: s.into() })
        }
    }
}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        download(self, args).map_err(|source| CmdError::Download { source })?;
        Ok(())
    }
}

impl Input {
    fn download_type(&self) -> &'static str {
        if self.original {
            "original"
        } else if self.archive {
            "archive"
        } else {
            "attachment"
        }
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

    #[snafu(display("{}", given))]
    InvalidDupeMode { given: String },
}

fn action_msg(opts: &Input, len: usize, target: Display) -> String {
    if opts.original {
        format!("original files of {} attachments into {}", len, target)
    } else if opts.archive {
        format!("archives of {} attachments into {}", len, target)
    } else {
        format!("{} attachments into {}", len, target)
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
                    "Zipping {}",
                    action_msg(opts, attachs.len(), zip_file.display())
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
                    "Downloading {}",
                    action_msg(opts, attachs.len(), parent.display())
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
        let (mut resp, url) = s.download(opts, args)?;
        if resp.status() == StatusCode::NOT_FOUND {
            println!("No {} file for attachment {}", opts.download_type(), s.name);
        } else {
            resp = resp.error_for_status().context(Http { url })?;
            let org_name = get_filename(&resp).unwrap_or(&s.name);
            let (fname, duplicate) = dupes.use_name(org_name);
            let path = parent.join(&fname);
            if path.exists() && !opts.overwrite {
                println!("File exists: {}. Skipping.", path.display());
            } else if duplicate && opts.dupes == DupeMode::Skip {
                println!("Skipping already downloaded file {}", org_name);
            } else {
                println!("Downloading {} …", &fname);
                let file = std::fs::File::create(path).context(CreateFile)?;
                let mut writer = std::io::BufWriter::new(file);
                resp.copy_to(&mut writer).context(ReadResponse)?;
            }
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
            let (mut resp, url) = s.download(opts, args)?;
            if resp.status() == StatusCode::NOT_FOUND {
                println!("No {} file for attachment {}", opts.download_type(), s.name);
            } else {
                resp = resp.error_for_status().context(Http { url })?;
                let org_name = get_filename(&resp).unwrap_or(&s.name);
                let (fname, duplicate) = dupes.use_name(org_name);
                if duplicate && opts.dupes == DupeMode::Skip {
                    println!("Skipping already downloaded file {}", org_name);
                } else {
                    zw.start_file(&fname, zip::write::FileOptions::default())
                        .context(Zip)?;
                    println!("Downloading {} …", &fname);
                    resp.copy_to(&mut zw).context(ReadResponse)?;
                }
            }
        }
        zw.finish().context(Zip)?;

        if dupes.is_empty() {
            match std::fs::remove_file(zip_file) {
                Ok(_) => log::info!("Emtpy zip file deleted."),
                Err(e) => log::warn!("Empty zip file could not be deleted! {}", e),
            }
        }
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
        offset: opts.offset,
        limit: opts.limit,
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

//////////////////////////////////////////////////////////////////////////////
/// Helper structs

#[derive(Debug)]
struct Attach {
    name: String,
    id: String,
}
impl Attach {
    fn to_url(&self, opts: &Input, args: &CmdArgs) -> String {
        let base = format!("{}/api/v1/sec/attachment/{}", args.docspell_url(), self.id);
        if opts.original {
            format!("{}/original", base)
        } else if opts.archive {
            format!("{}/archive", base)
        } else {
            base
        }
    }

    fn download(&self, opts: &Input, args: &CmdArgs) -> Result<(Response, String), Error> {
        let url = self.to_url(opts, args);
        let token = login::session_token(args).context(Login)?;
        let client = reqwest::blocking::Client::new();
        let resp = client
            .get(&url)
            .header(DOCSPELL_AUTH, &token)
            .send()
            //     .and_then(|r| r.error_for_status())
            .context(Http { url: url.clone() })?;
        Ok((resp, url))
    }
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

    fn use_name(&mut self, name: &str) -> (String, bool) {
        let fname = name.to_string();
        match self.names.get(&fname) {
            Some(count) => {
                let next_name = file::splice_name(name, count);
                let next_count = count + 1;
                self.names.insert(fname.clone(), next_count);
                (next_name, true)
            }
            None => {
                self.names.insert(fname.clone(), 1);
                (fname, false)
            }
        }
    }

    fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn unit_dupes_add() {
        let mut dupes = Dupes::new();
        assert_eq!(dupes.use_name("test.png"), ("test.png".into(), false));
        assert_eq!(dupes.use_name("test.png"), ("test_1.png".into(), true));
        assert_eq!(dupes.use_name("test.png"), ("test_2.png".into(), true));
        assert_eq!(dupes.use_name("test.png"), ("test_3.png".into(), true));
        assert_eq!(dupes.use_name("test.jpg"), ("test.jpg".into(), false));
    }
}
