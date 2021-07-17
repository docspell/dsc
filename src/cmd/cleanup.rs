use crate::opts::EndpointOpts;
use crate::{
    cmd::{Cmd, CmdArgs, CmdError},
    types::BasicResult,
};
use clap::{ArgGroup, Clap, ValueHint};
use snafu::{ResultExt, Snafu};
use std::path::PathBuf;

use super::file_exists;

/// Cleans directories from files that are in Docspell.
///
/// Traverses one or more directories and check each file whether it
/// exists in Docspell. If so, it can be deleted or moved to another
/// place.
#[derive(Clap, Debug)]
#[clap(group = ArgGroup::new("action"))]
pub struct Input {
    #[clap(flatten)]
    pub endpoint: EndpointOpts,

    /// Each file is moved into the given directory. The directory
    /// structure below the traversed directory is retained in the
    /// target.
    #[clap(long = "move", group = "action", value_hint = ValueHint::DirPath)]
    pub move_file: Option<PathBuf>,

    /// Each file is deleted.
    #[clap(short, long = "delete", group = "action")]
    pub delete_file: bool,

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

    #[snafu(display("Cannot delete: {}", source))]
    Delete { source: std::io::Error },

    #[snafu(display("Cannot move: {}", source))]
    Move { source: std::io::Error },

    #[snafu(display("No action given! Use --move or --delete."))]
    NoAction,

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
    match &args.move_file {
        Some(path) => {
            if path.is_dir() {
                Ok(())
            } else {
                Err(Error::TargetNotDirectory { path: path.clone() })
            }
        }
        None => {
            if args.delete_file {
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
                    counter = counter + cleanup_file(&cf, Some(&file), args, cfg)?;
                }
            }
        } else {
            counter = counter + cleanup_file(&file, None, args, cfg)?;
        }
    }
    Ok(counter)
}

fn cleanup_file(
    file: &PathBuf,
    root: Option<&PathBuf>,
    args: &Input,
    cfg: &CmdArgs,
) -> Result<u32, Error> {
    eprintln!("Check file: {}", file.display());
    let exists = check_file_exists(&file, &args.endpoint, cfg)?;
    log::debug!("Checking file: {} (exists: {})", file.display(), exists);
    if exists {
        eprint!(" - exists: ");
        if !args.dry_run {
            match &args.move_file {
                Some(target) => {
                    move_file(file, root, target)?;
                    eprintln!("moved.");
                    return Ok(1);
                }
                None => {
                    if args.delete_file {
                        delete_file(&file)?;
                        eprintln!("deleted.");
                        return Ok(1);
                    }
                }
            }
        } else {
            eprintln!("{}", exists);
            return Ok(1);
        }
    }
    Ok(0)
}

fn move_file(file: &PathBuf, root: Option<&PathBuf>, target: &PathBuf) -> Result<(), Error> {
    let target_file = match root {
        Some(r) => {
            let part = file.strip_prefix(r).unwrap();
            target.join(part)
        }
        None => target.join(file.file_name().unwrap()),
    };
    std::fs::rename(file, target_file).context(Move)
}

fn delete_file(file: &PathBuf) -> Result<(), Error> {
    std::fs::remove_file(file).context(Delete)
}

fn check_file_exists(path: &PathBuf, opts: &EndpointOpts, args: &CmdArgs) -> Result<bool, Error> {
    file_exists::check_file(path, opts, args)
        .context(FileExists)
        .map(|result| result.exists)
}
