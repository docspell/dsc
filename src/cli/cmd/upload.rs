use clap::{ArgGroup, Clap, ValueHint};
use snafu::{ResultExt, Snafu};
use std::path::PathBuf;

use super::{Cmd, Context};
use crate::cli::opts::{EndpointOpts, FileAction, UploadMeta};
use crate::cli::sink::Error as SinkError;
use crate::http::payload::{BasicResult, StringList, UploadMeta as MetaRequest};
use crate::http::Error as HttpError;
use crate::util::digest;
use crate::util::file::FileActionResult;

/// Uploads files to docspell.
///
/// To upload a file, an authenticated user is required, a source id
/// or the secret to the integration endpoint. The latter allows to
/// upload files to different collectives.
///
/// There are two modes for uploading: normal or via traversal.
/// Normally, it allows to upload multiple files via one single
/// request. Only files can be give as arguments. With `--traverse`
/// the arguments may be files or directories. If directories are
/// given, they are traversed recursively and each file is uploaded.
/// This mode creates one request per file. It is possible to specify
/// patters for inclusion/exclusion which apply to both modes. These
/// patterns are matched against the complete filename (with path).
///
/// Successfully uploaded files can be deleted or moved to another
/// directory. If a file already exists it is also deleted or moved.
/// So using `upload --traverse --delete` will upload all files that
/// are not yet in Docspell and then deletes them.
///
/// For glob patterns, see https://docs.rs/glob/0.3.0/glob/struct.Pattern.html
#[derive(Clap, Debug)]
#[clap(group = ArgGroup::new("g_multiple"))]
pub struct Input {
    #[clap(flatten)]
    pub endpoint: EndpointOpts,

    #[clap(flatten)]
    pub upload: UploadMeta,

    /// If set, all files are uploaded as one single item. Default is
    /// to create one item per file. This does not work with `--traverse`!
    #[clap(long = "single-item", parse(from_flag = std::ops::Not::not), group="g_multiple")]
    pub multiple: bool,

    /// A glob pattern for matching against each file. Note that
    /// usually you can just use the shells expansion mechanism.
    #[clap(long, short, default_value = "**/*")]
    pub matches: String,

    #[clap(flatten)]
    pub action: FileAction,

    /// A glob pattern that excludes files to upload. If `--matches`
    /// is also specified, both must evaluate to true.
    #[clap(long, short)]
    pub not_matches: Option<String>,

    /// Traverses directories and uploads all files that match the
    /// glob patterns if specfified. This means it cannot be used with
    /// `--multiple`, because each file is uploaded in a separate
    /// request. Without this, only files are accepted and uploaded in
    /// one single request.
    #[clap(long, short, group = "g_multiple")]
    pub traverse: bool,

    /// Can be used with `--traverse` to periodically run an upload.
    /// This option allows to set a delay in seconds that is waited in
    /// between runs. Please see the `watch` subcommand for a more
    /// efficient way to achieve the same.
    #[clap(long)]
    pub poll: Option<u64>,

    /// Doesn't submit the request, but prints which files would be
    /// uploaded instead. This might be useful when using `--traverse`
    /// and glob patterns.
    #[clap(long)]
    pub dry_run: bool,

    /// One or more files to upload
    #[clap(required = true, min_values = 1, value_hint = ValueHint::FilePath)]
    pub files: Vec<PathBuf>,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("The collective is required, but was not specified!"))]
    CollectiveNotGiven {},

    #[snafu(display("Unable to open file {}: {}", path.display(), source))]
    OpenFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Error creating hash for '{}': {}", path.display(), source))]
    DigestFile {
        source: std::io::Error,
        path: PathBuf,
    },

    #[snafu(display("Cannot delete or move {}: {}", path.display(), source))]
    FileActionError {
        source: std::io::Error,
        path: PathBuf,
    },

    #[snafu(display("Unable to read file at {}: {}", path.display(), source))]
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("An http error occurred: {}!", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },

    #[snafu(display("The `--single-item` option cannot be used with `--traverse`"))]
    MultipleWithTraverse,

    #[snafu(display("The `--poll` option requires `--traverse`"))]
    PollWithoutTraverse,

    #[snafu(display("The glob pattern '{}' is invalid: {}", pattern, source))]
    BadGlobPattern {
        source: glob::PatternError,
        pattern: String,
    },
}

impl Cmd for Input {
    type CmdError = Error;

    fn exec(&self, ctx: &Context) -> Result<(), Error> {
        let result = upload_files(self, ctx)?;
        ctx.write_result(result).context(WriteResult)?;
        Ok(())
    }
}

fn upload_files(args: &Input, ctx: &Context) -> Result<BasicResult, Error> {
    check_flags(args)?;
    let matcher = matching::Matcher::new(args)?;

    let meta = MetaRequest {
        multiple: args.multiple,
        direction: args
            .upload
            .direction
            .clone()
            .map(|d| d.to_value().to_string()),
        folder: args.upload.folder.clone(),
        skip_duplicates: args.upload.skip_duplicates,
        tags: StringList {
            items: args.upload.tag.clone(),
        },
        file_filter: args.upload.file_filter.clone(),
        language: args.upload.language.clone(),
    };
    log::debug!("Send file metadata: {:?}", serde_json::to_string(&meta));
    if args.traverse {
        if let Some(delay) = args.poll {
            let delay_dur = std::time::Duration::from_secs(delay);
            let dir_list = args
                .files
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<String>>()
                .join(", ");
            loop {
                eprintln!(
                    "Traversing to upload '{}' (every {:?}) …",
                    dir_list, delay_dur
                );
                upload_traverse(&meta, args, ctx, &matcher)?;
                std::thread::sleep(delay_dur);
            }
        } else {
            upload_traverse(&meta, args, ctx, &matcher)
        }
    } else {
        upload_single(&meta, args, ctx, matcher)
    }
}

fn apply_file_action(path: &PathBuf, root: Option<&PathBuf>, opts: &Input) -> Result<(), Error> {
    let res = opts
        .action
        .execute(path, root)
        .context(FileActionError { path })?;
    match res {
        FileActionResult::Deleted(_p) => {
            eprintln!("Deleted file");
            Ok(())
        }
        FileActionResult::Moved(p) => {
            eprintln!("Moved file to: {}", p.display());
            Ok(())
        }
        FileActionResult::Nothing => Ok(()),
    }
}

fn upload_traverse(
    meta: &MetaRequest,
    opts: &Input,
    ctx: &Context,
    matcher: &matching::Matcher,
) -> Result<BasicResult, Error> {
    let mut counter = 0;
    let fauth = opts.endpoint.to_file_auth(ctx);
    for path in &opts.files {
        if path.is_dir() {
            for child in matcher.traverse(&path)? {
                let exists = check_existence(&child, opts, ctx)?;
                if !exists {
                    eprintln!("Uploading {}", child.display());
                    counter = counter + 1;
                    if !opts.dry_run {
                        ctx.client
                            .upload_files(&fauth, meta, &vec![path])
                            .context(HttpClient)?;
                        apply_file_action(&child, Some(&path), opts)?;
                    }
                } else {
                    file_exists_message(&child);
                    apply_file_action(&child, Some(&path), opts)?;
                }
            }
        } else {
            if matcher.is_included(&path) {
                let exists = check_existence(path, opts, ctx)?;
                if !exists {
                    eprintln!("Uploading file {}", path.display());
                    counter = counter + 1;
                    if !opts.dry_run {
                        ctx.client
                            .upload_files(&fauth, meta, &vec![path])
                            .context(HttpClient)?;
                        apply_file_action(&path, None, opts)?;
                    }
                } else {
                    file_exists_message(path);
                    apply_file_action(&path, None, opts)?;
                }
            }
        }
    }

    Ok(BasicResult {
        success: true,
        message: format!("Uploaded {}", counter).into(),
    })
}

fn file_exists_message(path: &PathBuf) {
    eprintln!("File already in Docspell: {}", path.display());
}

/// Uploads all files in a single request.
fn upload_single(
    meta: &MetaRequest,
    opts: &Input,
    ctx: &Context,
    matcher: matching::Matcher,
) -> Result<BasicResult, Error> {
    let fauth = opts.endpoint.to_file_auth(ctx);
    let mut files: Vec<&PathBuf> = Vec::new();
    for path in &opts.files {
        if matcher.is_included(path) {
            let exists = check_existence(path, opts, ctx)?;
            if !exists {
                eprintln!("Adding to request: {}", path.display());
                if !opts.dry_run {
                    files.push(path);
                }
            } else {
                file_exists_message(path);
            }
        }
    }

    if !opts.dry_run {
        if files.len() > 0 {
            eprintln!("Sending request …");
            ctx.client
                .upload_files(&fauth, meta, &files)
                .context(HttpClient)
        } else {
            Ok(BasicResult {
                success: true,
                message: "No files to upload".into(),
            })
        }
    } else {
        Ok(BasicResult {
            success: true,
            message: format!("Would upload {} file(s)", files.len()).into(),
        })
    }
}

fn check_existence(path: &PathBuf, opts: &Input, ctx: &Context) -> Result<bool, Error> {
    if opts.upload.skip_duplicates {
        let fauth = opts.endpoint.to_file_auth(ctx);
        let hash = digest::digest_file_sha256(path).context(DigestFile { path })?;
        let exists = ctx.client.file_exists(hash, &fauth).context(HttpClient)?;
        Ok(exists.exists)
    } else {
        Ok(false)
    }
}

// TODO use clap to solve this!
fn check_flags(args: &Input) -> Result<(), Error> {
    if args.traverse && !args.multiple {
        return Err(Error::MultipleWithTraverse);
    }
    if args.poll.is_some() && !args.traverse {
        return Err(Error::PollWithoutTraverse);
    }

    Ok(())
}

//////////////////////////////////////////////////////////////////////////////
/// Helper types

mod matching {
    use super::*;
    use glob::{GlobResult, Paths, Pattern};

    pub struct Matcher {
        include: glob::Pattern,
        include_glob: String,
        exclude: Option<glob::Pattern>,
        exclude_glob: Option<String>,
    }

    impl Matcher {
        pub fn new(args: &Input) -> Result<Matcher, Error> {
            let include = glob::Pattern::new(&args.matches).context(BadGlobPattern {
                pattern: args.matches.clone(),
            })?;
            let exclude = match &args.not_matches {
                Some(nm) => Some(glob::Pattern::new(&nm).context(BadGlobPattern {
                    pattern: nm.to_string(),
                })?),
                None => None,
            };

            Ok(Matcher {
                include,
                include_glob: args.matches.clone(),
                exclude,
                exclude_glob: args.not_matches.clone(),
            })
        }

        pub fn is_included(&self, path: &PathBuf) -> bool {
            let bi = self.include.matches_path(path);
            let result = bi && !check_exclude(&self.exclude, path);
            log::debug!("Including '{}': {}", path.display(), result);
            result
        }

        pub fn traverse(&self, start: &PathBuf) -> Result<Matches, Error> {
            let pattern = start.join(&self.include_glob).display().to_string();
            log::info!("Traversing {} (excluding {:?})", pattern, self.exclude_glob);
            let paths = glob::glob(&pattern).context(BadGlobPattern {
                pattern: self.include_glob.clone(),
            })?;

            Ok(Matches {
                paths,
                excl: self.exclude.clone(),
            })
        }
    }

    pub struct Matches {
        paths: Paths,
        excl: Option<Pattern>,
    }

    impl Iterator for Matches {
        type Item = PathBuf;

        fn next(&mut self) -> Option<Self::Item> {
            loop {
                let next = self.paths.next();
                if next.is_none() {
                    return None;
                }
                let filtered = match next {
                    None => None,
                    Some(result) => match_result(result, &self.excl).filter(|p| p.is_file()),
                };
                if filtered.is_some() {
                    return filtered;
                } else {
                    continue;
                }
            }
        }
    }

    fn match_result(result: GlobResult, excl: &Option<Pattern>) -> Option<PathBuf> {
        match result {
            Ok(path) => {
                let is_excluded = check_exclude(excl, &path);
                log::debug!("Include file '{}': {}", path.display(), !is_excluded);
                if !is_excluded {
                    Some(path)
                } else {
                    None
                }
            }
            Err(err) => {
                log::error!("Cannot read file while traversing: {}", err);
                eprintln!("Skipping some entry while traversing due to error: {}", err);
                None
            }
        }
    }

    fn check_exclude(patt: &Option<glob::Pattern>, path: &PathBuf) -> bool {
        match patt {
            Some(p) => p.matches_path(path),
            None => false,
        }
    }
}
