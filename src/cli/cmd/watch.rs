use clap::{Parser, ValueHint};
use notify::{DebouncedEvent, RecursiveMode, Watcher};
use snafu::{ResultExt, Snafu};
use std::{path::Path, sync::mpsc};
use std::{path::PathBuf, time::Duration};

use super::{upload, Cmd, Context};
use crate::http::payload::BasicResult;
use crate::{
    cli::opts::{EndpointOpts, FileAction, UploadMeta},
    util::file::CollectiveSubdirErr,
};

use crate::util::file;

/// Watches a directory and uploads files to docspell.
///
/// It accepts the same authentication options as the `upload`
/// command.
///
/// When using the integration endpoint and a collective is not
/// specified, it will be guessed from the first subdirectory of the
/// directory that is specified.
///
/// On some filesystems, this command may not work (e.g. networking
/// file systems like NFS or SAMBA). You may use the `upload` command
/// then in combination with the `--poll` option.
#[derive(Parser, Debug)]
pub struct Input {
    /// Wether to watch directories recursively or not.
    #[clap(long, short)]
    pub recursive: bool,

    /// A delay in seconds after which the event is acted upon.
    #[clap(long = "delay", default_value = "6")]
    pub delay_secs: u64,

    #[clap(flatten)]
    pub upload: UploadMeta,

    #[clap(flatten)]
    pub action: FileAction,

    /// A glob pattern for matching against each file. Note that
    /// usually you can just use the shells expansion mechanism.
    #[clap(long, short, default_value = "**/*")]
    pub matches: String,

    /// A glob pattern that excludes files to upload. If `--matches`
    /// is also specified, both must evaluate to true.
    #[clap(long, short)]
    pub not_matches: Option<String>,

    /// Don't upload anything, but print what would be uploaded.
    #[clap(long)]
    pub dry_run: bool,

    #[clap(flatten)]
    pub endpoint: EndpointOpts,

    /// The directories to watch for changes.
    #[clap(value_hint = ValueHint::DirPath)]
    pub dirs: Vec<PathBuf>,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Uploading failed: {}", source))]
    Upload { source: upload::Error },

    #[snafu(display("Error creating hash for '{}': {}", path.display(), source))]
    DigestFile {
        source: std::io::Error,
        path: PathBuf,
    },

    #[snafu(display("Not a directory: {}", path.display()))]
    NotADirectory { path: PathBuf },

    #[snafu(display("Error while watching: {}", source))]
    Watch { source: notify::Error },

    #[snafu(display("Error consuming event: {}", source))]
    Event { source: mpsc::RecvError },

    #[snafu(display("Error finding collective: {}", source))]
    FindCollective { source: CollectiveSubdirErr },

    #[snafu(display("Could not find a collective for {}", path.display()))]
    NoCollective { path: PathBuf },
}

impl Cmd for Input {
    type CmdError = Error;

    fn exec(&self, ctx: &Context) -> Result<(), Error> {
        watch_directories(self, ctx)?;
        Ok(())
    }
}

pub fn watch_directories(opts: &Input, ctx: &Context) -> Result<(), Error> {
    check_is_dir(&opts.dirs)?;
    let mode = if opts.recursive {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    };
    let (tx, rx) = mpsc::channel();

    let mut watcher = notify::watcher(tx, Duration::from_secs(opts.delay_secs)).context(Watch)?;
    for dir in &opts.dirs {
        eprintln!("Watching directory ({:?}): {}", mode, dir.display());
        watcher.watch(dir, mode).context(Watch)?;
    }
    eprintln!("Press Ctrl-C to quit.");
    loop {
        match rx.recv() {
            Ok(event) => event_act(event, opts, ctx)?,
            Err(e) => return Err(Error::Event { source: e }),
        }
    }
}

fn check_is_dir(dirs: &[PathBuf]) -> Result<(), Error> {
    for path in dirs {
        if !path.is_dir() {
            return Err(Error::NotADirectory { path: path.clone() });
        }
    }
    Ok(())
}

fn event_act(event: DebouncedEvent, opts: &Input, ctx: &Context) -> Result<(), Error> {
    log::info!("Event: {:?}", event);
    match event {
        DebouncedEvent::Create(path) => upload_and_report(path, opts, ctx)?,
        DebouncedEvent::Write(path) => upload_and_report(path, opts, ctx)?,
        DebouncedEvent::Chmod(path) => upload_and_report(path, opts, ctx)?,
        DebouncedEvent::Error(err, path_opt) => {
            log::error!("Debounce event error for path {:?}: {}", path_opt, err);
            return Err(Error::Watch { source: err });
        }
        _ => (),
    }
    Ok(())
}

fn upload_and_report(path: PathBuf, opts: &Input, ctx: &Context) -> Result<(), Error> {
    eprintln!("------------------------------------------------------------------------------");
    eprintln!("Got: {}", path.display());
    let result = upload_file(path, opts, ctx)?;
    if result.success {
        if opts.dry_run {
            eprintln!("Dry run. Would upload now.");
        } else {
            eprintln!("Server: {}", result.message);
        }
    } else {
        log::error!("Error from uploading: {}", result.message);
        eprintln!("Sevrer Error: {}", result.message);
    }
    Ok(())
}

fn upload_file(path: PathBuf, opts: &Input, ctx: &Context) -> Result<BasicResult, Error> {
    let mut ep = opts.endpoint.clone();
    if let Some(cid) = find_collective(&path, &opts.dirs, &opts.endpoint)? {
        ep.collective = Some(cid);
    }

    let data = &upload::Input {
        endpoint: ep,
        multiple: true,
        action: opts.action.clone(),
        upload: opts.upload.clone(),
        matches: opts.matches.clone(),
        not_matches: opts.not_matches.clone(),
        traverse: false,
        poll: None,
        dry_run: opts.dry_run,
        files: vec![path],
    };
    upload::upload_files(data, ctx).context(Upload)
}

pub fn find_collective(
    path: &Path,
    dirs: &[PathBuf],
    opts: &EndpointOpts,
) -> Result<Option<String>, Error> {
    if opts.integration && opts.collective.is_none() {
        let cid = file::collective_from_subdir(path, dirs).context(FindCollective)?;
        if cid.is_none() {
            Err(Error::NoCollective {
                path: path.to_path_buf(),
            })
        } else {
            Ok(cid)
        }
    } else {
        Ok(None)
    }
}
