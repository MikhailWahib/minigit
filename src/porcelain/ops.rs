use anyhow::{Result, anyhow};
use std::{
    fs::{self, DirEntry, File, create_dir_all},
    io::Write,
    path::PathBuf,
};

use crate::{
    plumbing::object::ObjectId,
    plumbing::ops::{commit_tree, hash_object, update_index, write_tree},
    repository::Repository,
    utils,
};

pub fn init(repo: &Repository) -> Result<()> {
    let git_dir = repo.git_dir();
    create_dir_all(git_dir.join("objects"))?;
    create_dir_all(git_dir.join("refs/heads"))?;

    File::create_new(git_dir.join("refs/heads/master"))?;
    let mut head_file = File::create_new(git_dir.join("HEAD"))?;

    head_file.write_all(b"ref: refs/heads/master")?;
    Ok(())
}

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

pub fn commit(msg: String, repo: &Repository) -> Result<()> {
    let head_ref = fs::read(repo.git_dir().join("HEAD"))?;

    let branch_head = head_ref
        .strip_prefix(b"ref: ")
        .ok_or_else(|| anyhow!("error getting current branch head"))?;
    let branch_head = str::from_utf8(branch_head)?;

    let branch_head_path = repo.git_dir().join(branch_head);
    let parent_commit = fs::read(&branch_head_path)?;
    let parent_commit = str::from_utf8(&parent_commit)?;

    let parent_commit = if parent_commit.is_empty() {
        None
    } else {
        Some(ObjectId::from_hex(parent_commit)?)
    };

    let tree = write_tree(repo)?;
    let commit_hash = commit_tree(tree, parent_commit, msg, repo)?;

    fs::write(branch_head_path, commit_hash.to_hex())?;
    Ok(())
}
