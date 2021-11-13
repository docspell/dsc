use clap::{ArgEnum, ArgGroup, Parser};
use snafu::{ResultExt, Snafu};
use std::path::{Path, PathBuf};

use super::{Cmd, Context};
use crate::cli::opts::Format;
use crate::cli::sink::Error as SinkError;
use crate::cli::table::format_date_by;
use crate::http::payload::{Item, SearchMode, SearchReq};
use crate::http::{Downloads, Error as HttpError};
use crate::util::file;

#[derive(ArgEnum, Clone, Copy, Debug)]
pub enum LinkNaming {
    /// Name links to items after the items' id.
    Id,
    // Name links to items after the items' sanitized name.
    Name,
}
impl Default for LinkNaming {
    fn default() -> Self {
        LinkNaming::Id
    }
}

/// Exports data for a query.
///
/// Searches for documents via a query and downloads all associated
/// files and metadata. It downloads the original file and not the
/// converted one.
///
/// Use the `search-summary` command with the same query to get an
/// idea how much is being downloaded.
///
/// This commands creates a specific directory structure in the
/// `target` directory. All files are stored below the `items`
/// subdirectory. In there the first two letters of the item id are
/// used to create another subdirectory. Then the complete item id is
/// used for another subdirectory. In the last one, a file
/// `metadata.json` is created that contains all the metadata to the
/// item (tags, correspondents, etc). The attachments are all stored
/// in the `files` subdirectory.
///
/// The `--*-links` options can be used to create a symlink tree based
/// on some metadata, like tags, correspondents or item date.
#[derive(Parser, std::fmt::Debug)]
#[clap(group = ArgGroup::new("kind"))]
pub struct Input {
    /// Limit the number of results.
    #[clap(short, long, default_value = "100")]
    limit: u32,

    /// Skip the first n results.
    #[clap(short, long, default_value = "0")]
    offset: u32,

    /// If `true`, all entries are exported. That is, the `offset` is
    /// incremented until all entries have been exported.
    #[clap(short, long)]
    all: bool,

    /// Overwrite already existing files. By default the download is
    /// skipped if there is already a file with the same name present.
    #[clap(long)]
    overwrite: bool,

    /// Specify after which of an items' property the links to it
    /// should be named. (Defaults to id)
    #[clap(long, arg_enum)]
    link_naming: Option<LinkNaming>,

    /// Creates symlinks by item date. This may not work on some file
    /// systems.
    #[clap(long)]
    date_links: bool,

    /// Create symlinks by tag. This may not work on some file
    /// systems.
    #[clap(long)]
    tag_links: bool,

    /// Create symlinks by folder. This may not work on some
    /// file systems.
    #[clap(long)]
    folder_links: bool,

    /// Create symlinks by correspondent. This may not work on some
    /// file systems.
    #[clap(long)]
    correspondent_links: bool,

    /// If your Folder-names contain a custom delimiter used to represent
    /// flat hierarchy (e.g. "Financial/Invoices"), the delimiter you set
    /// with this option is used to split the Folder name into a path, which
    /// is then created on the file-system when using the folder-links export.
    #[clap(long)]
    folder_delimiter: Option<String>,

    /// Download everything into this directory.
    #[clap(short, long)]
    target: PathBuf,

    /// The optional query string. If not given everything is
    /// exported. See https://docspell.org/docs/query/
    query: Option<String>,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An http error occurred: {}!", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },

    #[snafu(display("Error creating json: {}", source))]
    Json { source: serde_json::Error },

    #[snafu(display("Error creating a file: {}", source))]
    CreateFile { source: std::io::Error },

    #[snafu(display("Error deleting a file: {}", source))]
    DeleteFile { source: std::io::Error },

    #[snafu(display("Error creating a symlink: {}", source))]
    Symlink { source: std::io::Error },

    #[snafu(display("Not a directory: {}", path.display()))]
    NotADirectory { path: PathBuf },
}

impl Cmd for Input {
    type CmdError = Error;

    fn exec(&self, ctx: &Context) -> Result<(), Error> {
        let mut req = SearchReq {
            offset: self.offset,
            limit: self.limit,
            with_details: true,
            query: self.query.clone().unwrap_or_else(|| "".into()),
            search_mode: SearchMode::Normal,
        };

        let mut counter = 0;
        loop {
            let next = export(&req, self, ctx)?;
            counter += next;
            if self.all && next >= self.limit as usize {
                req.offset += req.limit;
            } else {
                break;
            }
        }
        eprintln!("Exported {} items.", counter);
        Ok(())
    }
}

fn export(req: &SearchReq, opts: &Input, ctx: &Context) -> Result<usize, Error> {
    let results = ctx
        .client
        .search(&ctx.opts.session, req)
        .context(HttpClient)?;
    let mut item_counter = 0;
    let items = opts.target.join("items");
    let by_date = opts.target.join("by_date");
    let by_tag = opts.target.join("by_tag");
    let by_folder = opts.target.join("by_folder");
    let by_corr = opts.target.join("by_correspondent");
    for g in results.groups {
        for item in g.items {
            item_counter += 1;
            let item_dir = items.join(&item.id[0..2]).join(&item.id);
            export_item(&item, opts.overwrite, &item_dir, ctx)?;

            if opts.date_links {
                let link_dir = by_date.join(format_date_by(item.date, "%Y-%m"));
                make_links(&item, opts, &item_dir, &link_dir)?;
            }
            if opts.correspondent_links {
                let corr_opt = item.corr_org.as_ref().or_else(|| item.corr_person.as_ref());
                if let Some(corr) = corr_opt {
                    let link_dir = by_corr.join(file::safe_filename(&corr.name));
                    make_links(&item, opts, &item_dir, &link_dir)?;
                }
            }
            if opts.tag_links {
                for tag in &item.tags {
                    let link_dir = by_tag.join(file::safe_filename(&tag.name));
                    make_links(&item, opts, &item_dir, &link_dir)?;
                }
            }
            if opts.folder_links {
                let folder_opt = item
                    .folder
                    .as_ref()
                    .map(|f| file::safe_filepath(&f.name, &opts.folder_delimiter));
                if let Some(folder_name) = folder_opt {
                    let link_dir = by_folder.join(folder_name);
                    make_links(&item, opts, &item_dir, &link_dir)?;
                }
            }
            export_message(item, ctx)?;
        }
    }
    Ok(item_counter)
}

fn export_message(item: Item, ctx: &Context) -> Result<(), Error> {
    match ctx.format() {
        Format::Tabular => eprintln!("Exported item: {}", item.name),
        Format::Csv => eprintln!("Exported item: {}", item.name),
        _ => ctx.write_result(item).context(WriteResult)?,
    }

    Ok(())
}

fn export_item(item: &Item, overwrite: bool, item_dir: &Path, ctx: &Context) -> Result<(), Error> {
    log::debug!("Exporting item {}/{}", item.id, item.name);
    let meta_file = item_dir.join("metadata.json");
    if meta_file.exists() && overwrite {
        log::debug!(
            "Remove existing meta file {}, due to overwrite=true",
            meta_file.display()
        );
        std::fs::remove_file(&meta_file).context(DeleteFile)?;
    }
    if !item_dir.exists() {
        std::fs::create_dir_all(&item_dir).context(CreateFile)?;
    }
    if !&meta_file.exists() {
        let file = std::fs::File::create(&meta_file).context(CreateFile)?;
        let fw = std::io::BufWriter::new(file);
        serde_json::to_writer_pretty(fw, item).context(Json)?;
    } else {
        log::debug!("Skip existing meta file: {}", meta_file.display());
    }

    let file_dir = item_dir.join("files");
    if !file_dir.exists() {
        std::fs::create_dir_all(&file_dir).context(CreateFile)?;
    }
    let dl = Downloads::from_item(item);
    for attach in dl {
        log::debug!("Saving attachment: {}/{}", attach.id, attach.name);
        let orig = attach
            .get_original(&ctx.client, &ctx.opts.session)
            .context(HttpClient)?;
        if let Some(mut orig_file) = orig {
            let file_name = orig_file
                .get_filename()
                .unwrap_or_else(|| attach.name.as_str());
            let file_path = file_dir.join(file_name);
            if file_path.exists() && overwrite {
                log::debug!(
                    "Removing existing {}, due to overwrite=true",
                    file_path.display()
                );
                std::fs::remove_file(&meta_file).context(DeleteFile)?;
            }
            if !file_path.exists() {
                let file = std::fs::File::create(&file_path).context(CreateFile)?;
                let mut fw = std::io::BufWriter::new(file);
                orig_file.copy_to(&mut fw).context(HttpClient)?;
            } else {
                log::debug!("Skipping existing file {}", file_path.display());
            }
        }
    }
    Ok(())
}

fn make_links(
    item: &Item,
    opts: &Input,
    link_target: &Path,
    link_name_path: &Path,
) -> Result<(), Error> {
    if !link_name_path.exists() {
        std::fs::create_dir_all(&link_name_path).context(CreateFile)?;
    }
    let link_filename = match opts.link_naming.unwrap_or_default() {
        LinkNaming::Id => item.id.clone(),
        LinkNaming::Name => file::safe_filename(&item.name),
    };

    let rel_link_target = pathdiff::diff_paths(&link_target, &link_name_path).unwrap();
    // Append the item's id as link name on the link's path.
    let mut link_name = link_name_path.join(&link_filename);
    let mut collision_counter = 1;
    let create_link = loop {
        // Use read_link() instead of exists(), because the latter traverses links and instead
        // checks whether the link-target exists.
        let link_data = link_name.read_link();
        match link_data {
            // A link with this name already exists.
            Ok(link_data) => {
                if link_data == rel_link_target {
                    // This link is pointing to the item we want to create a link for
                    // skip depending on whether the "overwrite" property is set
                    break opts.overwrite;
                } else {
                    // this is simply a name collision (same name, different document).
                    // append a number to the name, to remove conflict, then try again
                    log::debug!("Found name collision for: \"{}\"", link_name.display());
                    link_name =
                        link_name_path.join(format!("{} ({})", link_filename, collision_counter));
                    collision_counter += 1;
                }
            }
            // Link does not yet exist, all good, we can have a go
            _ => {
                break true;
            }
        }
    };

    if create_link {
        file::symlink(rel_link_target, link_name).context(Symlink)?;
    } else {
        log::debug!("Skip existing link: {}", link_target.display());
    }
    Ok(())
}
