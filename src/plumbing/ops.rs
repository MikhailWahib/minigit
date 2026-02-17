use anyhow::{Context, Result, anyhow};
use std::fs;
use std::io;
use std::path::Path;

use super::core::{format_tree, hash_content, read_object, write_object, write_tree_recursive};
use super::index::Index;
use super::object::{ObjectId, ObjectType};
use crate::plumbing::commit::Commit;
use crate::plumbing::index_tree::IndexTree;
use crate::repository::Repository;

pub fn hash_object(object_path: &str, write: bool, repo: &Repository) -> Result<ObjectId> {
    let content = fs::read(object_path)
        .with_context(|| format!("Failed to open object at {}", object_path))?;

    if !write {
        return Ok(hash_content(&content, ObjectType::Blob));
    }

    let objects_dir = repo.git_dir().join("objects");
    write_object(&content, ObjectType::Blob, &objects_dir)
}

pub fn cat_file(object_id: ObjectId, typ: bool, repo: &Repository) -> Result<String> {
    let objects_dir = repo.git_dir().join("objects");
    let (obj_type, body) = read_object(&object_id, &objects_dir)?;

    if typ {
        return Ok(obj_type.to_string());
    }

    if obj_type == ObjectType::Tree {
        let formatted = format_tree(&body)?;
        return Ok(formatted);
    }

    let body_str = std::str::from_utf8(&body).unwrap_or("<binary>");
    Ok(body_str.to_string())
}

pub fn ls_files(repo: &Repository) -> Result<Option<String>> {
    let idx_path = repo.git_dir().join("index");

    match Index::read(idx_path) {
        Ok(idx) => Ok(Some(idx.to_string())),
        Err(e)
            if e.downcast_ref::<io::Error>()
                .is_some_and(|e| e.kind() == io::ErrorKind::NotFound) =>
        {
            // ignore missing index
            Ok(None)
        }
        Err(e) => Err(e),
    }
}

pub fn update_index(mode: u32, object: ObjectId, file: String, repo: &Repository) -> Result<()> {
    let idx_path = repo.git_dir().join("index");
    let mut idx = match Index::read(&idx_path) {
        Ok(i) => i,
        Err(e)
            if e.downcast_ref::<io::Error>()
                .is_some_and(|e| e.kind() == io::ErrorKind::NotFound) =>
        {
            Index::new()
        }
        Err(e) => return Err(e),
    };

    idx.add(file, object.as_bytes(), mode)?;
    idx.write(idx_path)?;

    Ok(())
}

pub fn remove_from_inedx(file: String, repo: &Repository) -> Result<()> {
    let idx_path = repo.git_dir().join("index");
    let mut idx = match Index::read(&idx_path) {
        Ok(i) => i,
        Err(e)
            if e.downcast_ref::<io::Error>()
                .is_some_and(|e| e.kind() == io::ErrorKind::NotFound) =>
        {
            Index::new()
        }
        Err(e) => return Err(e),
    };

    idx.remove(file);
    idx.write(idx_path)?;

    Ok(())
}

pub fn write_tree(repo: &Repository) -> Result<ObjectId> {
    let idx = Index::read(repo.git_dir().join("index"))?;
    let entries = idx.entries();
    let idx_dir_tree = IndexTree::from_idx_entries(entries);

    let objects_dir = repo.git_dir().join("objects");
    write_tree_recursive(&idx_dir_tree, &objects_dir)
}

pub fn commit_tree(
    tree: ObjectId,
    parent: Option<ObjectId>,
    message: String,
    repo: &Repository,
) -> Result<ObjectId> {
    let objects_dir = repo.git_dir().join("objects");
    validate_object_reference(&tree, ObjectType::Tree, &objects_dir)?;

    if let Some(parent_hash) = parent {
        validate_object_reference(&parent_hash, ObjectType::Commit, &objects_dir)?;
    }

    let commit = Commit::new(tree, parent, message)?;
    write_object(&commit.to_bytes(), ObjectType::Commit, &objects_dir)
}

fn validate_object_reference(
    object: &ObjectId,
    expected_type: ObjectType,
    objects_dir: &Path,
) -> Result<()> {
    let (obj_type, _) =
        read_object(object, objects_dir).with_context(|| format!("object not found: {object}"))?;
    if obj_type != expected_type {
        return Err(anyhow!(
            "invalid object type for {object}: expected {expected_type}, got {obj_type}"
        ));
    }

    Ok(())
}
