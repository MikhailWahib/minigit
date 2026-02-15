use anyhow::Result;
use std::{fs::DirEntry, path::PathBuf};

pub fn walk_dir(entry: &DirEntry) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if entry.file_type()?.is_file() {
        files.push(entry.path());
    } else {
        let mut sub = Vec::new();
        let children = entry.path().read_dir()?;
        for c in children {
            let c = c?;
            sub.append(&mut walk_dir(&c)?)
        }
        return Ok(sub);
    }
    Ok(files)
}
