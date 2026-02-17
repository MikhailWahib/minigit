use anyhow::{Result, anyhow, bail};
use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};

use crate::{
    plumbing::{index::Index, object::ObjectId, ops::cat_file},
    porcelain::utils,
    repository::Repository,
};

pub type StagedChange = (String, String);

pub struct StatusChanges {
    pub untracked: Vec<String>,
    pub modified: Vec<String>,
    pub staged: Vec<StagedChange>,
}

pub fn status_changes(repo: &Repository) -> Result<StatusChanges> {
    let worktree = repo.work_tree();
    let ignore = repo.get_ignored()?;

    let mut worktree_files = Vec::new();
    utils::collect_files(worktree, &ignore, &mut worktree_files)?;

    let mut worktree_files: Vec<String> = worktree_files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let mut untracked = Vec::new();
    let mut modified = Vec::new();
    let mut staged = Vec::new();

    let Some(idx) = read_index(repo)? else {
        // if no index found, all worktree is untracked
        untracked.append(&mut worktree_files);
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
        let rel_path = file
            .strip_prefix(worktree.to_str().unwrap())
            .unwrap()
            .trim_start_matches('/');

        if let Some(idx_entry) = idx.get(rel_path) {
            if idx_entry.is_modified(repo)? {
                modified.push(rel_path.to_string());
            }
        } else {
            untracked.push(rel_path.to_string());
        }
    }

    Ok(StatusChanges {
        untracked,
        modified,
        staged,
    })
}

pub fn has_staged_changes(repo: &Repository) -> Result<bool> {
    Ok(!status_changes(repo)?.staged.is_empty())
}

pub fn head_ref_path(repo: &Repository) -> Result<PathBuf> {
    let head_ref = fs::read(repo.git_dir().join("HEAD"))?;
    let branch_head = head_ref
        .strip_prefix(b"ref: ")
        .ok_or_else(|| anyhow!("error getting current branch head"))?;
    let branch_head = str::from_utf8(branch_head)?.trim();
    Ok(repo.git_dir().join(branch_head))
}

pub fn current_branch(repo: &Repository) -> Result<String> {
    let path = head_ref_path(repo)?;
    let git_path = repo.git_dir();

    let branch = path
        .strip_prefix(format!(
            "{}/refs/heads/",
            git_path.to_string_lossy().to_string()
        ))?
        .to_str()
        .ok_or_else(|| anyhow!("Malformed ref"))?;

    Ok(branch.to_owned())
}

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

fn read_index(repo: &Repository) -> Result<Option<Index>> {
    let idx_path = repo.git_dir().join("index");
    Index::read_optional(idx_path)
}

fn get_head_tree_entries(repo: &Repository) -> Result<BTreeMap<String, [u8; 20]>> {
    let mut entries = BTreeMap::new();
    let Some(commit_id) = read_head_commit(repo)? else {
        return Ok(entries);
    };

    if cat_file(commit_id, true, repo)? != "commit" {
        bail!("HEAD does not point to a commit object");
    }

    let commit_body = cat_file(commit_id, false, repo)?;
    let tree_hex = commit_body
        .lines()
        .find_map(|line| line.strip_prefix("tree "))
        .ok_or_else(|| anyhow!("Malformed commit: missing tree header"))?;
    let tree_id = ObjectId::from_hex(tree_hex)?;

    collect_tree_entries(repo, tree_id, Path::new(""), &mut entries)?;
    Ok(entries)
}

fn collect_tree_entries(
    repo: &Repository,
    tree_id: ObjectId,
    prefix: &Path,
    entries: &mut BTreeMap<String, [u8; 20]>,
) -> Result<()> {
    if cat_file(tree_id, true, repo)? != "tree" {
        bail!("Expected tree object while traversing HEAD tree");
    }

    let tree_listing = cat_file(tree_id, false, repo)?;
    for line in tree_listing.lines() {
        let (meta, name) = line
            .split_once('\t')
            .ok_or_else(|| anyhow!("Invalid tree listing line: missing tab separator"))?;

        let mut meta_parts = meta.split_whitespace();
        let _mode = meta_parts
            .next()
            .ok_or_else(|| anyhow!("Invalid tree listing line: missing mode"))?;
        let entry_type = meta_parts
            .next()
            .ok_or_else(|| anyhow!("Invalid tree listing line: missing type"))?;
        let sha_hex = meta_parts
            .next()
            .ok_or_else(|| anyhow!("Invalid tree listing line: missing object id"))?;

        let sha = ObjectId::from_hex(sha_hex)?;
        let path = prefix.join(name);
        let path_str = path.to_string_lossy().to_string();

        if entry_type == "tree" {
            collect_tree_entries(repo, sha, &path, entries)?;
        } else {
            entries.insert(path_str, sha.as_bytes());
        }
    }

    Ok(())
}
