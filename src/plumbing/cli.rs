use anyhow::Result;

use crate::repository::Repository;

use super::ops;

pub fn hash_object(object_path: &str, write: bool, repo: &Repository) -> Result<String> {
    ops::hash_object(object_path, write, repo)
}

pub fn cat_file(hash: &str, typ: bool, repo: &Repository) -> Result<String> {
    ops::cat_file(hash, typ, repo)
}

pub fn ls_files(repo: &Repository) -> Result<Option<String>> {
    ops::ls_files(repo)
}

pub fn update_index(mode: &str, object: &str, path: String, repo: &Repository) -> Result<()> {
    let mode_u32 = mode.parse::<u32>()?;
    ops::update_index(mode_u32, object, path, repo)
}

pub fn write_tree(repo: &Repository) -> Result<String> {
    ops::write_tree(repo)
}

pub fn commit_tree(
    tree: String,
    parent: Option<String>,
    message: String,
    repo: &Repository,
) -> Result<String> {
    ops::commit_tree(tree, parent, message, repo)
}
