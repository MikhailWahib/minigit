use anyhow::{Result, anyhow};
use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};

use crate::{
    plumbing::{
        index::Index,
        object::{ObjectId, ObjectType},
        ops::{read_commit, read_tree},
    },
    porcelain::utils,
    repository::Repository,
};

pub type StagedChange = (String, String);

pub struct StatusChanges {
    pub untracked: Vec<String>,
    pub modified: Vec<String>,
    pub staged: Vec<StagedChange>,
}

/// Computes untracked, modified, and staged changes for the current repository state.
pub fn status_changes(repo: &Repository) -> Result<StatusChanges> {
    let worktree = repo.work_tree();
    let ignore = repo.get_ignored()?;

    let mut worktree_files = Vec::new();
    utils::collect_files(worktree, &ignore, &mut worktree_files)?;

    let mut untracked = Vec::new();
    let mut modified = Vec::new();
    let mut staged = Vec::new();

    let Some(idx) = read_index(repo)? else {
        // if no index found, all worktree is untracked
        for file in &worktree_files {
            untracked.push(path_to_rel_string(file, worktree)?);
        }
        return Ok(StatusChanges {
            untracked,
            modified,
            staged,
        });
    };

    let index_map = idx.entries_map();

    let head_tree_map = get_head_tree_entries(repo)?;

    // check for staged files by comparing index with HEAD tree
    // if a file exists in the index but not in the HEAD tree -> new file
    // if a file exists in both but with different hash -> modified file
    for (path, entry) in index_map {
        match head_tree_map.get(path) {
            None => staged.push(("new file".to_string(), path.clone())),
            Some(head_sha) if *head_sha != entry.sha1 => {
                staged.push(("modified".to_string(), path.clone()))
            }
            Some(_) => {}
        }
    }
    // if a file exists in HEAD tree but not the index -> deleted file
    for path in head_tree_map.keys() {
        if !index_map.contains_key(path) {
            staged.push(("deleted".to_string(), path.clone()));
        }
    }

    // check for modified (unstaged) and untracked files
    for file in worktree_files {
        let rel_path = path_to_rel_string(&file, worktree)?;

        if let Some(idx_entry) = idx.get(&rel_path) {
            if idx_entry.is_modified(repo)? {
                modified.push(rel_path);
            }
        } else {
            untracked.push(rel_path);
        }
    }

    Ok(StatusChanges {
        untracked,
        modified,
        staged,
    })
}

/// Returns true when at least one staged change exists.
pub fn has_staged_changes(repo: &Repository) -> Result<bool> {
    Ok(!status_changes(repo)?.staged.is_empty())
}

/// Resolves the branch reference path pointed to by `HEAD`.
pub fn head_ref_path(repo: &Repository) -> Result<PathBuf> {
    let head_ref = fs::read(repo.git_dir().join("HEAD"))?;
    let branch_head = head_ref
        .strip_prefix(b"ref: ")
        .ok_or_else(|| anyhow!("error getting current branch head"))?;
    let branch_head = str::from_utf8(branch_head)?.trim();
    Ok(repo.git_dir().join(branch_head))
}

/// Returns the current branch name derived from `HEAD`.
pub fn current_branch(repo: &Repository) -> Result<String> {
    let path = head_ref_path(repo)?;
    let branch = path
        .strip_prefix(repo.git_dir().join("refs/heads"))?
        .to_str()
        .ok_or_else(|| anyhow!("Malformed ref"))?;

    Ok(branch.to_owned())
}

/// Reads the commit pointed to by the current branch head, if present.
pub fn read_head_commit(repo: &Repository) -> Result<Option<ObjectId>> {
    let branch_head_path = head_ref_path(repo)?;
    let branch_commit = match fs::read_to_string(&branch_head_path) {
        Ok(content) => content,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };

    let branch_commit = branch_commit.trim();
    if branch_commit.is_empty() {
        return Ok(None);
    }

    Ok(Some(ObjectId::from_hex(branch_commit)?))
}

/// Reads the index file if it exists.
fn read_index(repo: &Repository) -> Result<Option<Index>> {
    let idx_path = repo.git_dir().join("index");
    Index::read_optional(idx_path)
}

/// Collects a map of file paths to blob object ids from the HEAD tree.
fn get_head_tree_entries(repo: &Repository) -> Result<BTreeMap<String, [u8; 20]>> {
    let mut entries = BTreeMap::new();
    let Some(commit_id) = read_head_commit(repo)? else {
        return Ok(entries);
    };

    let commit = read_commit(commit_id, repo)?;
    let tree_id = commit.tree();

    collect_tree_entries(repo, tree_id, Path::new(""), &mut entries)?;
    Ok(entries)
}

/// Recursively walks a tree object and flattens blob entries into `entries`.
fn collect_tree_entries(
    repo: &Repository,
    tree_id: ObjectId,
    prefix: &Path,
    entries: &mut BTreeMap<String, [u8; 20]>,
) -> Result<()> {
    for entry in read_tree(tree_id, repo)? {
        let path = prefix.join(&entry.name);
        let path_str = path.to_string_lossy().to_string();

        if entry.object_type == ObjectType::Tree {
            collect_tree_entries(repo, entry.object_id, &path, entries)?;
        } else {
            entries.insert(path_str, entry.object_id.as_bytes());
        }
    }

    Ok(())
}

/// Converts an absolute path under `worktree` into a UTF-8 relative string path.
fn path_to_rel_string(path: &Path, worktree: &Path) -> Result<String> {
    let relative = path.strip_prefix(worktree)?;
    let value = relative
        .to_str()
        .ok_or_else(|| anyhow!("File path is not valid UTF-8"))?;
    Ok(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::path_to_rel_string;
    use std::path::Path;

    #[test]
    fn converts_worktree_path_to_relative_string() {
        let worktree = Path::new("/tmp/repo");
        let file = Path::new("/tmp/repo/src/main.rs");

        let rel = path_to_rel_string(file, worktree).expect("relative path");
        assert_eq!(rel, "src/main.rs");
    }

    #[test]
    fn fails_for_path_outside_worktree() {
        let worktree = Path::new("/tmp/repo");
        let file = Path::new("/tmp/other/file.txt");

        let err = path_to_rel_string(file, worktree).expect_err("must fail");
        assert!(
            format!("{err}").contains("prefix"),
            "unexpected error: {err}"
        );
    }
}
