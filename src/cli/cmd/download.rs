use clap::{ArgGroup, Parser, ValueEnum};
use snafu::{ResultExt, Snafu};
use std::path::{Display, Path, PathBuf};

use super::{Cmd, Context};
use crate::http::payload::SearchReq;
use crate::{
    cli::opts::SearchMode,
    http::{Downloads, Error as HttpError},
    util::dupes::Dupes,
};

/// Downloads files given a query.
///
/// Searches for documents via a query and downloads all associated
/// files. It downloads by default the converted PDF files, which can
/// be changed using options `--original` and `--archive`,
/// respectively.
///
/// Use the `search-summary` command with the same query to get an
/// idea how much is being downloaded.
#[derive(Parser, std::fmt::Debug)]
#[command(group = ArgGroup::new("kind"))]
pub struct Input {
    /// The query string. See <https://docspell.org/docs/query/>
    query: String,

    #[clap(flatten)]
    pub search_mode: SearchMode,

    /// Limit the number of results.
    #[arg(short, long, default_value = "60")]
    limit: u32,

    /// Skip the first n results.
    #[arg(short, long, default_value = "0")]
    offset: u32,

    /// Whether to overwrite already existing files. By default the
    /// download is skipped if there is already a file with the target
    /// name present. When using `--zip` this will remove an existing
    /// zip file before downloading.
    #[arg(long)]
    overwrite: bool,

    /// Download the original file instead of the converted PDF.
    #[arg(long, group = "kind")]
    original: bool,

    /// Download the original archive file to the attachment if
    /// available. Since often multiple files map to a single archive,
    /// the option `--dupes skip` can be used here.
    #[arg(long, group = "kind")]
    archive: bool,

    /// Creates a single zip file containing all files (flat). If this
    /// is enabled, the `target` option is expected to be the target
    /// zip file and not a directory.
    #[arg(long)]
    zip: bool,

    /// What to do when multiple files map to the same name. Can be
    /// one of: skip, rename. For rename, the target file is renamed
    /// by appending a number suffix.
    #[arg(long, value_enum, default_value = "rename")]
    dupes: DupeMode,

    /// Download everything into this directory. If not given, the
    /// current working directory is used. If `--zip` is used, this is
    /// the zip file to create.
    #[arg(short, long)]
    target: Option<PathBuf>,
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

#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum DupeMode {
    Skip,
    Rename,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An http error occurred: {}", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error creating a file. {}", source))]
    CreateFile { source: std::io::Error },

    #[snafu(display("Error creating zip file. {}", source))]
    Zip { source: zip::result::ZipError },

    #[snafu(display("{}", given))]
    InvalidDupeMode { given: String },

    #[snafu(display("Not a directory: {}", path.display()))]
    NotADirectory { path: PathBuf },

    #[snafu(display("Not a file: {}", path.display()))]
    NotAFile { path: PathBuf },
}

impl Cmd for Input {
    type CmdError = Error;

    fn exec(&self, ctx: &Context) -> Result<(), Error> {
        check_args(self)?;
        let req = SearchReq {
            offset: self.offset,
            limit: self.limit,
            with_details: true,
            query: self.query.clone(),
            search_mode: self.search_mode.to_mode(),
        };
        let attachs = ctx
            .client
            .download_search(&ctx.opts.session, &req)
            .context(HttpClientSnafu)?;

        if attachs.is_empty() {
            println!("The search result is empty.");
            Ok(())
        } else {
            match self.zip {
                true => {
                    let zip_file = self
                        .target
                        .clone()
                        .unwrap_or_else(|| PathBuf::from("docspell-files.zip"));
                    if let Some(parent) = zip_file.parent() {
                        if !parent.exists() {
                            std::fs::create_dir_all(&parent).context(CreateFileSnafu)?;
                        }
                    }
                    println!(
                        "Zipping {}",
                        action_msg(self, attachs.len(), zip_file.display())
                    );

                    download_zip(attachs, self, ctx, &zip_file)
                }
                false => {
                    let parent = self
                        .target
                        .clone()
                        .unwrap_or(std::env::current_dir().context(CreateFileSnafu)?);

                    if !parent.exists() {
                        std::fs::create_dir_all(&parent).context(CreateFileSnafu)?;
                    }
                    println!(
                        "Downloading {}",
                        action_msg(self, attachs.len(), parent.display())
                    );

                    download_flat(attachs, self, ctx, &parent)
                }
            }
        }
    }
}

fn download_flat(
    attachs: Downloads,
    opts: &Input,
    ctx: &Context,
    parent: &Path,
) -> Result<(), Error> {
    let mut dupes = Dupes::new();
    for dref in attachs {
        let dlopt = if opts.original {
            dref.get_original(&ctx.client, &ctx.opts.session)
        } else if opts.archive {
            dref.get_archive(&ctx.client, &ctx.opts.session)
        } else {
            dref.get(&ctx.client, &ctx.opts.session)
        }
        .context(HttpClientSnafu)?;

        if let Some(mut dl) = dlopt {
            let org_name = dl.get_filename().unwrap_or(dref.name);
            let (fname, duplicate) = dupes.use_name(&org_name);
            let path = parent.join(&fname);
            if path.exists() && !opts.overwrite {
                println!("File exists: {}. Skipping.", path.display());
            } else if duplicate && opts.dupes == DupeMode::Skip {
                println!("Skipping already downloaded file {}", org_name);
            } else {
                println!("Downloading {} …", &fname);
                let file = std::fs::File::create(path).context(CreateFileSnafu)?;
                let mut writer = std::io::BufWriter::new(file);
                dl.copy_to(&mut writer).context(HttpClientSnafu)?;
            }
        } else {
            println!(
                "No {} file for attachment {}",
                opts.download_type(),
                dref.name
            );
        }
    }
    Ok(())
}

fn download_zip(
    attachs: Downloads,
    opts: &Input,
    ctx: &Context,
    zip_file: &Path,
) -> Result<(), Error> {
    if zip_file.exists() && !opts.overwrite {
        println!("Zip file already exists! {}", zip_file.display());
    } else {
        if zip_file.exists() {
            std::fs::remove_file(zip_file).context(CreateFileSnafu)?;
        }
        let zip = std::fs::File::create(zip_file).context(CreateFileSnafu)?;
        let mut zw = zip::ZipWriter::new(zip);
        let mut dupes = Dupes::new();
        for dref in attachs {
            let dlopt = if opts.original {
                dref.get_original(&ctx.client, &ctx.opts.session)
            } else if opts.archive {
                dref.get_archive(&ctx.client, &ctx.opts.session)
            } else {
                dref.get(&ctx.client, &ctx.opts.session)
            }
            .context(HttpClientSnafu)?;

            if let Some(mut dl) = dlopt {
                let org_name = dl.get_filename().unwrap_or(dref.name);
                let (fname, duplicate) = dupes.use_name(&org_name);
                if duplicate && opts.dupes == DupeMode::Skip {
                    println!("Skipping already downloaded file {}", org_name);
                } else {
                    zw.start_file(&fname, zip::write::FileOptions::default())
                        .context(ZipSnafu)?;
                    println!("Downloading {} …", &fname);
                    dl.copy_to(&mut zw).context(HttpClientSnafu)?;
                }
            } else {
                println!(
                    "No {} file for attachment {}",
                    opts.download_type(),
                    dref.name
                );
            }
        }
        zw.finish().context(ZipSnafu)?;

        if dupes.is_empty() {
            match std::fs::remove_file(zip_file) {
                Ok(_) => log::info!("Emtpy zip file deleted."),
                Err(e) => log::warn!("Empty zip file could not be deleted! {}", e),
            }
        }
    }
    Ok(())
}

fn check_args(args: &Input) -> Result<(), Error> {
    match &args.target {
        Some(path) => {
            if args.zip && path.exists() && path.is_dir() {
                Err(Error::NotAFile { path: path.clone() })
            } else if !args.zip && !path.is_dir() && path.exists() {
                Err(Error::NotADirectory { path: path.clone() })
            } else {
                Ok(())
            }
        }
        None => Ok(()),
    }
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
