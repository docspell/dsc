use crate::cmd::{Cmd, CmdArgs, CmdError};
use crate::opts::{EndpointOpts, UploadMeta};
use crate::types::{BasicResult, StringList, UploadMeta as MetaRequest, DOCSPELL_AUTH};
use clap::{ArgGroup, Clap, ValueHint};
use reqwest::blocking::multipart::{Form, Part};
use reqwest::blocking::RequestBuilder;
use snafu::{ResultExt, Snafu};
use std::fs::File;
use std::path::PathBuf;

use super::file_exists;
use super::login;

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
/// For glob patterns, see https://docs.rs/glob/0.3.0/glob/struct.Pattern.html
#[derive(Clap, Debug)]
#[clap(group = ArgGroup::new("g_multiple"))]
pub struct Input {
    #[clap(flatten)]
    pub endpoint: EndpointOpts,

    #[clap(flatten)]
    pub upload: UploadMeta,

    /// A glob pattern for matching against each file. Note that
    /// usually you can just use the shells expansion mechanism.
    #[clap(long, short, default_value = "**/*")]
    pub matches: String,

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

    /// Doesn't submit the request, but prints which files would be
    /// uploaded instead. This might be useful when using `--traverse`
    /// and glob patterns.
    #[clap(long)]
    pub dry_run: bool,

    /// One or more files to upload
    #[clap(required = true, min_values = 1, value_hint = ValueHint::FilePath)]
    pub files: Vec<PathBuf>,
}
impl Input {
    fn collective_id(&self) -> Result<&String, Error> {
        self.endpoint
            .collective
            .as_ref()
            .ok_or(Error::CollectiveNotGiven {})
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("The collective is required, but was not specified!"))]
    CollectiveNotGiven {},

    #[snafu(display("Serializing the upload meta data failed!"))]
    MetaSerialize { source: serde_json::Error },

    #[snafu(display("Unable to create the upload part: {}", source))]
    PartCreate { source: reqwest::Error },

    #[snafu(display("Unable to open file {}: {}", path.display(), source))]
    OpenFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Unable to read file at {}: {}", path.display(), source))]
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Error received from server at {}: {}", url, source))]
    Http { source: reqwest::Error, url: String },

    #[snafu(display("Error received from server: {}", source))]
    ReadResponse { source: reqwest::Error },

    #[snafu(display(
        "Error logging in via session. Consider the `login` command. {}",
        source
    ))]
    Login { source: login::Error },

    #[snafu(display("The `--single-item` option cannot be used with `--traverse`"))]
    MultipleWithTraverse,

    #[snafu(display("The glob pattern '{}' is invalid: {}", pattern, source))]
    BadGlobPattern {
        source: glob::PatternError,
        pattern: String,
    },

    #[snafu(display("Error while detecting if file exists! {}", source))]
    FileExists { source: file_exists::Error },
}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        let result = upload_files(self, args).map_err(|source| CmdError::Upload { source })?;
        args.write_result(result)?;

        Ok(())
    }
}

pub fn upload_files(args: &Input, cfg: &CmdArgs) -> Result<BasicResult, Error> {
    check_flags(args)?;
    let matcher = matching::Matcher::new(args)?;

    let url = &if args.endpoint.integration {
        let coll_id = args.collective_id()?;
        format!(
            "{}/api/v1/open/integration/item/{}",
            cfg.cfg.docspell_url, coll_id
        )
    } else {
        match &args.endpoint.get_source_id(cfg.cfg) {
            Some(id) => format!("{}/api/v1/open/upload/item/{}", cfg.docspell_url(), id),
            None => format!("{}/api/v1/sec/upload/item", cfg.docspell_url()),
        }
    };

    let meta = &MetaRequest {
        multiple: args.upload.multiple,
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
    let meta_json = serde_json::to_vec(&meta).context(MetaSerialize)?;
    log::debug!("Send file metadata: {:?}", serde_json::to_string(&meta));
    if args.traverse {
        upload_traverse(url, meta_json, args, cfg, matcher)
    } else {
        let meta_part = Part::bytes(meta_json)
            .mime_str(APP_JSON)
            .context(PartCreate)?;

        upload_single(url, meta_part, args, cfg, matcher)
    }
}

fn upload_traverse(
    url: &str,
    meta_json: Vec<u8>,
    args: &Input,
    cfg: &CmdArgs,
    matcher: matching::Matcher,
) -> Result<BasicResult, Error> {
    let mut counter = 0;
    for path in &args.files {
        if path.is_dir() {
            for child in matcher.traverse(&path)? {
                let exists = check_existence(&child, args, cfg)?;
                if !exists {
                    eprintln!("Uploading {}", child.display());
                    counter = counter + 1;
                    if !args.dry_run {
                        send_file(url, &child, meta_json.clone(), args, cfg)?;
                    }
                } else {
                    file_exists_message(&child);
                }
            }
        } else {
            if matcher.is_included(&path) {
                let exists = check_existence(path, args, cfg)?;
                if !exists {
                    eprintln!("Uploading file {}", path.display());
                    counter = counter + 1;
                    if !args.dry_run {
                        send_file(url, path, meta_json.clone(), args, cfg)?;
                    }
                } else {
                    file_exists_message(path);
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

fn send_file(
    url: &str,
    path: &PathBuf,
    meta_json: Vec<u8>,
    args: &Input,
    cfg: &CmdArgs,
) -> Result<BasicResult, Error> {
    let meta_part = Part::bytes(meta_json)
        .mime_str(APP_JSON)
        .context(PartCreate)?;
    let mut form = Form::new().part("meta", meta_part);
    let fopen = File::open(path).context(OpenFile { path })?;
    let len = fopen.metadata().context(OpenFile { path })?.len();
    let bufr = std::io::BufReader::new(fopen);
    let mut fpart = Part::reader_with_length(bufr, len);
    if let Some(fname) = path.as_path().file_name() {
        let f: String = fname.to_string_lossy().into();
        fpart = fpart.file_name(f);
    }
    form = form.part("file", fpart);
    send_file_form(form, url, args, cfg)
}

/// Uploads all files in a single request.
fn upload_single(
    url: &str,
    meta_part: Part,
    args: &Input,
    cfg: &CmdArgs,
    matcher: matching::Matcher,
) -> Result<BasicResult, Error> {
    let mut form = Form::new().part("meta", meta_part);
    let mut counter = 0;
    for path in &args.files {
        if matcher.is_included(path) {
            let exists = check_existence(path, args, cfg)?;
            if !exists {
                counter = counter + 1;
                eprintln!("Adding to request: {}", path.display());
                if !args.dry_run {
                    let fopen = File::open(path).context(OpenFile { path })?;
                    let len = fopen.metadata().context(OpenFile { path })?.len();
                    let bufr = std::io::BufReader::new(fopen);
                    let mut fpart = Part::reader_with_length(bufr, len);
                    if let Some(fname) = path.as_path().file_name() {
                        let f: String = fname.to_string_lossy().into();
                        fpart = fpart.file_name(f);
                    }
                    form = form.part("file", fpart);
                }
            } else {
                file_exists_message(path);
            }
        }
    }

    if !args.dry_run {
        if counter > 0 {
            eprintln!("Sending request …");
            send_file_form(form, url, args, cfg)
        } else {
            Ok(BasicResult {
                success: true,
                message: "No files to upload".into(),
            })
        }
    } else {
        Ok(BasicResult {
            success: true,
            message: format!("Would upload {} file(s)", &counter).into(),
        })
    }
}

fn check_existence(path: &PathBuf, args: &Input, cfg: &CmdArgs) -> Result<bool, Error> {
    if args.upload.skip_duplicates {
        file_exists::check_file(path, &args.endpoint, cfg)
            .context(FileExists)
            .map(|result| result.exists)
    } else {
        Ok(false)
    }
}

fn send_file_form(
    form: Form,
    url: &str,
    args: &Input,
    cfg: &CmdArgs,
) -> Result<BasicResult, Error> {
    let client = create_client(&url, &args.endpoint, cfg)?;
    client
        .multipart(form)
        .send()
        .and_then(|r| r.error_for_status())
        .context(Http { url })?
        .json::<BasicResult>()
        .context(ReadResponse)
}

// TODO use clap to solve this!
fn check_flags(args: &Input) -> Result<(), Error> {
    if args.traverse && !args.upload.multiple {
        Err(Error::MultipleWithTraverse)
    } else {
        Ok(())
    }
}

fn create_client(url: &str, opts: &EndpointOpts, args: &CmdArgs) -> Result<RequestBuilder, Error> {
    if opts.get_source_id(args.cfg).is_none() && !opts.integration {
        let token = login::session_token(args).context(Login)?;
        Ok(args.client.post(url).header(DOCSPELL_AUTH, token))
    } else {
        let mut c = args.client.post(url);
        c = opts.apply(c);
        Ok(c)
    }
}

//////////////////////////////////////////////////////////////////////////////
/// Helper types

const APP_JSON: &'static str = "application/json";

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
