use std::path::{Path, PathBuf};

use crate::cli::opts::FileAction;
use std::io;

/// Puts `suffix` in the filename before the extension.
pub fn splice_name(fname: &str, suffix: &i32) -> String {
    let p = PathBuf::from(fname);

    match p.extension() {
        Some(ext) => {
            let mut base = fname.trim_end_matches(ext.to_str().unwrap()).chars();
            base.next_back();
            format!("{}_{}.{}", base.as_str(), suffix, ext.to_str().unwrap())
        }
        None => format!("{}_{}", fname, suffix),
    }
}

fn delete_parent_if_empty(file: &Path, root: Option<&PathBuf>) -> Result<(), std::io::Error> {
    match (root, file.parent()) {
        (Some(r), Some(p)) => {
            if p != r && std::fs::read_dir(p)?.next().is_none() {
                std::fs::remove_dir(p)
            } else {
                Ok(())
            }
        }
        _ => Ok(()),
    }
}

pub fn collective_from_subdir(
    path: &Path,
    roots: &[PathBuf],
) -> Result<Option<String>, std::path::StripPrefixError> {
    let file = path.canonicalize().unwrap();
    for dir in roots {
        let can_dir = dir.canonicalize().unwrap();
        log::debug!("Check prefix {} -> {}", can_dir.display(), file.display());
        if file.starts_with(&can_dir) {
            let rest = file.strip_prefix(&can_dir)?;
            let coll = rest.iter().next();
            log::debug!("Found collective: {:?}", &coll);
            return Ok(coll.and_then(|s| s.to_str()).map(|s| s.to_string()));
        }
    }
    Ok(None)
}

pub fn safe_filename(name: &str) -> String {
    name.replace("/", "-")
}

#[cfg(windows)]
pub fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) -> io::Result<()> {
    std::os::windows::fs::symlink_dir(original, link)
}

#[cfg(unix)]
pub fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) -> io::Result<()> {
    std::os::unix::fs::symlink(original, link)
}

#[derive(Debug, Clone)]
pub enum FileActionResult {
    Deleted(PathBuf),
    Moved(PathBuf),
    Nothing,
}

impl FileAction {
    pub fn execute(
        &self,
        file: &Path,
        root: Option<&PathBuf>,
    ) -> Result<FileActionResult, std::io::Error> {
        match &self.move_to {
            Some(target) => Self::move_file(file, root, target).map(FileActionResult::Moved),
            None => {
                if self.delete {
                    Self::delete_file(&file, root)
                        .map(|_| FileActionResult::Deleted(file.to_path_buf()))
                } else {
                    Ok(FileActionResult::Nothing)
                }
            }
        }
    }

    fn move_file(
        file: &Path,
        root: Option<&PathBuf>,
        target: &Path,
    ) -> Result<PathBuf, std::io::Error> {
        let target_file = match root {
            Some(r) => {
                let part = file.strip_prefix(r).unwrap();
                target.join(part)
            }
            None => target.join(file.file_name().unwrap()),
        };
        log::debug!(
            "Move file '{}' -> '{}'",
            file.display(),
            &target_file.display()
        );
        if let Some(parent) = &target_file.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::fs::rename(file, &target_file)?;
        // delete the parent when below root. if no root given, don't delete parent
        delete_parent_if_empty(file, root)?;
        Ok(target_file)
    }

    fn delete_file(file: &Path, root: Option<&PathBuf>) -> Result<(), std::io::Error> {
        log::debug!("Deleting file: {}", file.display());
        std::fs::remove_file(file)?;
        delete_parent_if_empty(file, root)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_splice_name() {
        assert_eq!(splice_name("abc.pdf", &1), "abc_1.pdf");
        assert_eq!(splice_name("abc", &1), "abc_1");
        assert_eq!(splice_name("stuff.tar.gz", &2), "stuff.tar_2.gz");
    }
}
