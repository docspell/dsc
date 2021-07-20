use crate::{
    cmd::{Cmd, CmdArgs, CmdError},
    types::BasicResult,
};
use crate::{
    file::FileActionResult,
    opts::{EndpointOpts, FileAction},
};
use clap::Clap;
use snafu::{ResultExt, Snafu};
use std::path::PathBuf;

use super::file_exists;
use super::watch;

/// Cleans directories from files that are in Docspell.
///
/// Traverses one or more directories and check each file whether it
/// exists in Docspell. If so, it can be deleted or moved to another
/// place.
///
/// If you want to upload all files that don't exists in some
/// directory, use the `upload` command.
#[derive(Clap, Debug)]
pub struct Input {
    #[clap(flatten)]
    pub endpoint: EndpointOpts,

    #[clap(flatten)]
    pub action: FileAction,

    /// Each file is printed.
    #[clap(long)]
    pub dry_run: bool,

    /// One or more files/directories to check. Directories are
    /// traversed recursively.
    #[clap(required = true, min_values = 1)]
    pub files: Vec<PathBuf>,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error on file exists: {}", source))]
    FileExists { source: file_exists::Error },

    #[snafu(display("Pattern error: {}", source))]
    Pattern { source: glob::PatternError },

    #[snafu(display("Glob error: {}", source))]
    Glob { source: glob::GlobError },

    #[snafu(display("Cannot delete or move: {}", source))]
    FileActionError { source: std::io::Error },

    #[snafu(display("No action given! Use --move or --delete."))]
    NoAction,

    #[snafu(display("A collective was not found and was not specified"))]
    NoCollective,

    #[snafu(display("The target '{}' is not a directory!", path.display()))]
    TargetNotDirectory { path: PathBuf },
}

impl Cmd for Input {
    fn exec(&self, args: &CmdArgs) -> Result<(), CmdError> {
        let result = check_args(self)
            .and_then(|_unit| cleanup(self, args))
            .map_err(|source| CmdError::Cleanup { source })?;
        args.write_result(BasicResult {
            success: true,
            message: format!("Cleaned up files: {}", result).into(),
        })?;
        Ok(())
    }
}

fn check_args(args: &Input) -> Result<(), Error> {
    match &args.action.move_to {
        Some(path) => {
            if path.is_dir() {
                Ok(())
            } else {
                Err(Error::TargetNotDirectory { path: path.clone() })
            }
        }
        None => {
            if args.action.delete {
                Ok(())
            } else {
                Err(Error::NoAction)
            }
        }
    }
}

pub fn cleanup(args: &Input, cfg: &CmdArgs) -> Result<u32, Error> {
    let mut counter = 0;
    for file in &args.files {
        if file.is_dir() {
            let pattern = file.join("**/*").display().to_string();
            for child in glob::glob(&pattern).context(Pattern)? {
                let cf = child.context(Glob)?;
                if cf.is_file() {
                    counter = counter + cleanup_and_report(&cf, Some(&file), args, cfg)?;
                }
            }
        } else {
            counter = counter + cleanup_and_report(&file, None, args, cfg)?;
        }
    }
    Ok(counter)
}

fn cleanup_and_report(
    file: &PathBuf,
    root: Option<&PathBuf>,
    args: &Input,
    cfg: &CmdArgs,
) -> Result<u32, Error> {
    eprintln!("Check file: {}", file.display());
    let exists = check_file_exists(&file, root, &args.endpoint, cfg)?;
    log::debug!("Checking file: {} (exists: {})", file.display(), exists);
    if exists {
        eprint!(" - exists: ");
        if !args.dry_run {
            let res = args.action.execute(file, root).context(FileActionError)?;
            log::debug!("Action executed: {:?}", res);
            match res {
                FileActionResult::Deleted(_p) => {
                    eprintln!("deleted.");
                    return Ok(1);
                }
                FileActionResult::Moved(_p) => {
                    eprintln!("moved.");
                    return Ok(1);
                }
                FileActionResult::Nothing => {
                    log::error!("No file action defined. This should not happen, because user was able to not define it");
                    return Ok(0);
                }
            }
        } else {
            eprintln!("{}", exists);
            return Ok(1);
        }
    }
    Ok(0)
}

fn check_file_exists(
    path: &PathBuf,
    root: Option<&PathBuf>,
    opts: &EndpointOpts,
    args: &CmdArgs,
) -> Result<bool, Error> {
    let mut ep = opts.clone();
    let dirs: Vec<PathBuf> = match root {
        Some(d) => vec![d.clone()],
        None => vec![],
    };
    if let Some(cid) =
        watch::find_collective(path, &dirs, opts).map_err(|_e| Error::NoCollective)?
    {
        ep.collective = Some(cid);
    }
    file_exists::check_file(path, &ep, args)
        .context(FileExists)
        .map(|result| result.exists)
}
