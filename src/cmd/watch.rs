use std::{path::PathBuf, time::Duration};

use crate::{
    cmd::{Cmd, CmdArgs, CmdError},
    opts::{EndpointOpts, UploadMeta},
    types::BasicResult,
};
use clap::{Clap, ValueHint};
use notify::{DebouncedEvent, RecursiveMode, Watcher};
use snafu::{ResultExt, Snafu};
use std::sync::mpsc;

use super::upload;

/// Watches a directory and uploads files to docspell.
#[derive(Clap, Debug)]
pub struct Input {
    /// Wether to watch directories recursively or not.
    #[clap(long, short)]
    recursive: bool,

    /// A delay in seconds after which the event is acted upon.
    #[clap(long = "delay", default_value = "6")]
    delay_secs: u64,

    #[clap(flatten)]
    upload: UploadMeta,

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
    dry_run: bool,

    #[clap(flatten)]
    pub endpoint: EndpointOpts,

    /// The directories to watch for changes.
    #[clap(value_hint = ValueHint::DirPath)]
    dirs: Vec<PathBuf>,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Not a directory: {}!", path.display()))]
    NotADirectory { path: PathBuf },

    #[snafu(display("Error uploading: {}", source))]
    Upload { source: upload::Error },

    #[snafu(display("Error while watching: {}", source))]
    Watch { source: notify::Error },

    #[snafu(display("Error consuming event: {}", source))]
    Event { source: mpsc::RecvError },
}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        watch_directories(self, args).map_err(|source| CmdError::Watch { source })?;
        Ok(())
    }
}

pub fn watch_directories(opts: &Input, args: &CmdArgs) -> Result<(), Error> {
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
            Ok(event) => event_act(event, opts, args)?,
            Err(e) => return Err(Error::Event { source: e }),
        }
    }
}

fn check_is_dir(dirs: &Vec<PathBuf>) -> Result<(), Error> {
    for path in dirs {
        if !path.is_dir() {
            return Err(Error::NotADirectory { path: path.clone() });
        }
    }
    Ok(())
}

fn event_act(event: DebouncedEvent, opts: &Input, args: &CmdArgs) -> Result<(), Error> {
    log::info!("Event: {:?}", event);
    match event {
        DebouncedEvent::Create(path) => upload_and_report(path, opts, args)?,
        DebouncedEvent::Write(path) => upload_and_report(path, opts, args)?,
        DebouncedEvent::Chmod(path) => upload_and_report(path, opts, args)?,
        DebouncedEvent::Error(err, path_opt) => {
            log::error!("Debounce event error for path {:?}: {}", path_opt, err);
            return Err(Error::Watch { source: err });
        }
        _ => (),
    }
    Ok(())
}

fn upload_and_report(path: PathBuf, opts: &Input, args: &CmdArgs) -> Result<(), Error> {
    eprintln!("------------------------------------------------------------------------------");
    eprintln!("Got: {}", path.display());
    let result = upload_file(path, opts, args)?;
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

fn upload_file(path: PathBuf, opts: &Input, args: &CmdArgs) -> Result<BasicResult, Error> {
    let data = &upload::Input {
        endpoint: opts.endpoint.clone(),
        multiple: true,
        upload: opts.upload.clone(),
        matches: opts.matches.clone(),
        not_matches: opts.not_matches.clone(),
        traverse: false,
        dry_run: opts.dry_run,
        files: vec![path],
    };
    upload::upload_files(data, args).context(Upload)
}
