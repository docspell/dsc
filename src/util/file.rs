use std::path::PathBuf;

use crate::cli::opts::FileAction;

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

fn delete_parent_if_empty(file: &PathBuf, root: Option<&PathBuf>) -> Result<(), std::io::Error> {
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

#[derive(Debug, Clone)]
pub enum FileActionResult {
    Deleted(PathBuf),
    Moved(PathBuf),
    Nothing,
}

impl FileAction {
    pub fn execute(
        &self,
        file: &PathBuf,
        root: Option<&PathBuf>,
    ) -> Result<FileActionResult, std::io::Error> {
        match &self.move_to {
            Some(target) => Self::move_file(file, root, target).map(|p| FileActionResult::Moved(p)),
            None => {
                if self.delete {
                    Self::delete_file(&file, root).map(|_r| FileActionResult::Deleted(file.clone()))
                } else {
                    Ok(FileActionResult::Nothing)
                }
            }
        }
    }

    fn move_file(
        file: &PathBuf,
        root: Option<&PathBuf>,
        target: &PathBuf,
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

    fn delete_file(file: &PathBuf, root: Option<&PathBuf>) -> Result<(), std::io::Error> {
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
    fn unit_filename_from_header() {
        assert_eq!(
            filename_from_header("inline; filename=\"test.jpg\""),
            Some("test.jpg")
        );
    }

    #[test]
    fn unit_splice_name() {
        assert_eq!(splice_name("abc.pdf", &1), "abc_1.pdf");
        assert_eq!(splice_name("abc", &1), "abc_1");
        assert_eq!(splice_name("stuff.tar.gz", &2), "stuff.tar_2.gz");
    }
}
