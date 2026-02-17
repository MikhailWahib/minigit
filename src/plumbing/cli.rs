use crate::repository::Repository;
use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};

use super::object::ObjectId;
use super::ops;

pub fn hash_object(object_path: &Path, write: bool, repo: &Repository) -> Result<()> {
    let resolved_path = repo.resolve_from_cwd(object_path);
    let hash = ops::hash_object(&resolved_path, write, repo).map(|id| id.to_string())?;
    println!("{hash}");
    Ok(())
}

pub fn cat_file(hash: &str, typ: bool, repo: &Repository) -> Result<()> {
    let object_id = ObjectId::from_hex(hash)?;
    let output = ops::cat_file(object_id, typ, repo)?;
    if typ {
        println!("{output}");
    } else {
        print!("{output}");
    }

    Ok(())
}

pub fn ls_files(repo: &Repository) -> Result<()> {
    if let Some(output) = ops::ls_files(repo)? {
        print!("{output}");
    }

    Ok(())
}

pub fn update_index(mode: &str, object: &str, path: PathBuf, repo: &Repository) -> Result<()> {
    let mode_u32 = mode.parse::<u32>()?;
    let object_id = ObjectId::from_hex(object)?;
    let path = path
        .to_str()
        .ok_or_else(|| anyhow!("Path is not valid UTF-8"))?
        .to_string();
    ops::update_index(mode_u32, object_id, path, repo)
}

pub fn write_tree(repo: &Repository) -> Result<String> {
    ops::write_tree(repo).map(|id| id.to_string())
}

pub fn commit_tree(
    tree: String,
    parent: Option<String>,
    message: String,
    repo: &Repository,
) -> Result<()> {
    let tree_id = ObjectId::from_hex(&tree)?;
    let parent_id = parent.as_deref().map(ObjectId::from_hex).transpose()?;

    let hash = ops::commit_tree(tree_id, parent_id, message, repo).map(|id| id.to_string())?;

    println!("{hash}");

    Ok(())
}
