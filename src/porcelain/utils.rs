use anyhow::Result;
use std::{
    fs::DirEntry,
    path::{Path, PathBuf},
};

pub fn collect_files(path: &Path, ignore: &[PathBuf], files: &mut Vec<PathBuf>) -> Result<()> {
    if is_ignored(path, ignore) {
        return Ok(());
    }

    if path.is_file() {
        files.push(path.to_path_buf());
        return Ok(());
    }

    if path.is_dir() {
        let entries: Vec<DirEntry> = path.read_dir()?.flatten().collect();
        for entry in entries {
            collect_files(&entry.path(), ignore, files)?;
        }
    }

    Ok(())
}

fn is_ignored(path: &Path, ignore: &[PathBuf]) -> bool {
    ignore
        .iter()
        .any(|ignored| path == ignored || (ignored.is_dir() && path.starts_with(ignored)))
}
