use anyhow::{Result, anyhow};
use std::{fs::DirEntry, path::PathBuf};

use crate::{
    plumbing::ops::{hash_object, update_index},
    repository::Repository,
    utils,
};

pub fn add(paths: Vec<String>, repo: &Repository) -> Result<()> {
    let worktree = repo.work_tree();
    let ignore = repo.get_ignored()?;

    let dir_entries: Vec<DirEntry> = paths
        .iter()
        .flat_map(|p| worktree.join(p).read_dir().into_iter().flatten())
        .flatten()
        .filter(|p| !ignore.contains(&p.path()))
        .collect();

    let mut files: Vec<PathBuf> = Vec::new();
    for entry in &dir_entries {
        files.append(&mut utils::walk_dir(entry)?);
    }

    for file in files {
        let file_path = file
            .strip_prefix(repo.work_tree().to_string_lossy().to_string())?
            .to_str()
            .ok_or_else(|| anyhow!("File path is not valid UTF-8"))?;

        let object_id = hash_object(file_path, true, repo)?;
        update_index(100644, object_id, file_path.to_string(), repo)?;
    }

    Ok(())
}
