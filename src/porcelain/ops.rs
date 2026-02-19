use anyhow::{Result, anyhow, bail};
use std::{
    fs::{self, File, create_dir_all},
    io::Write,
    path::PathBuf,
};

use crate::{
    plumbing::{
        self,
        ops::{commit_tree, hash_object, update_index_batch, write_tree},
    },
    porcelain::{core, utils},
    repository::Repository,
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

pub fn add(paths: Vec<PathBuf>, repo: &Repository) -> Result<()> {
    let ignore = repo.get_ignored()?;

    let mut files: Vec<PathBuf> = Vec::new();
    for path in &paths {
        let path = repo.resolve_from_cwd(path);
        utils::collect_files(&path, &ignore, &mut files)?;
    }

    let mut staged_entries = Vec::with_capacity(files.len());
    for file in files {
        let file_path = file
            .strip_prefix(repo.work_tree())?
            .to_str()
            .ok_or_else(|| anyhow!("File path is not valid UTF-8"))?;

        let object_id = hash_object(&file, true, repo)?;
        staged_entries.push((100644, object_id, file_path.to_string()));
    }
    update_index_batch(staged_entries, repo)?;

    Ok(())
}

pub fn commit(msg: String, repo: &Repository) -> Result<()> {
    if !core::has_staged_changes(repo)? {
        bail!("no changes added to commit (use \"minigit add\" and/or \"minigit commit -a\")");
    }

    let branch_head_path = core::head_ref_path(repo)?;
    let parent_commit = core::read_head_commit(repo)?;

    let tree = write_tree(repo)?;
    let commit_hash = commit_tree(tree, parent_commit, msg, repo)?;

    fs::write(branch_head_path, commit_hash.to_hex())?;
    Ok(())
}

pub fn status(repo: &Repository) -> Result<(Vec<String>, Vec<String>, Vec<(String, String)>)> {
    let changes = core::status_changes(repo)?;
    Ok((changes.untracked, changes.modified, changes.staged))
}

pub fn remove(paths: Vec<PathBuf>, repo: &Repository) -> Result<()> {
    let worktree_path = repo.work_tree();
    let ignore = repo.get_ignored()?;
    let mut files = Vec::new();

    for p in &paths {
        let p = repo.resolve_from_cwd(p);
        utils::collect_files(&p, &ignore, &mut files)?;
    }

    let files: Vec<String> = files
        .into_iter()
        .filter_map(|p| {
            p.strip_prefix(worktree_path)
                .ok()?
                .to_str()
                .map(|s| s.to_string())
        })
        .collect();

    plumbing::ops::remove_from_index_batch(files, repo)?;

    Ok(())
}

pub fn log(
    repo: &Repository,
) -> Result<Vec<(plumbing::object::ObjectId, plumbing::commit::Commit)>> {
    let branch = core::current_branch(repo)?;
    let mut commits = Vec::new();
    let Some(head_commit) = core::read_head_commit(repo)? else {
        bail!("your current branch '{branch}' does not have any commits yet")
    };

    let mut current = Some(head_commit);
    while let Some(commit_id) = current {
        let commit = plumbing::ops::read_commit(commit_id, repo)?;
        current = commit.parent();
        commits.push((commit_id, commit));
    }

    Ok(commits)
}
