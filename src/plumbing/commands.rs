use anyhow::{Context, Result, anyhow};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use super::core::{format_tree, hash_content, read_object, write_object, write_tree_recursive};
use super::index::Index;
use super::object::{ObjectId, ObjectType};
use crate::plumbing::commit::Commit;
use crate::plumbing::index_tree::IndexTree;
use crate::repository::Repository;

pub fn hash_object(object_path: &str, write: bool, repo: &Repository) -> Result<String> {
    let mut file = File::open(&object_path)
        .with_context(|| format!("Failed to open object at {}", object_path))?;

    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    if !write {
        return Ok(hash_content(&content, ObjectType::Blob).to_string());
    }

    let objects_dir = repo.git_dir().join("objects");
    let object_id = write_object(&content, ObjectType::Blob, &objects_dir)?;
    Ok(object_id.to_string())
}

pub fn cat_file(hash: &str, typ: bool, repo: &Repository) -> Result<String> {
    let object_id = ObjectId::from_hex(hash)?;
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

pub fn update_index(mode: u32, object: &str, file: String, repo: &Repository) -> Result<()> {
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

    let object_id = ObjectId::from_hex(object)?;

    idx.add(file, object_id.as_bytes(), mode)?;
    idx.write(idx_path)?;

    Ok(())
}

pub fn write_tree(repo: &Repository) -> Result<String> {
    let idx = Index::read(repo.git_dir().join("index"))?;
    let entries = idx.entries();
    let idx_dir_tree = IndexTree::from_idx_entries(entries);

    let objects_dir = repo.git_dir().join("objects");
    let root_hash = write_tree_recursive(&idx_dir_tree, &objects_dir)?;

    Ok(root_hash.to_string())
}

pub fn commit_tree(
    tree: String,
    parent: Option<String>,
    message: String,
    repo: &Repository,
) -> Result<String> {
    let objects_dir = repo.git_dir().join("objects");
    let tree_id = ObjectId::from_hex(&tree)?;
    validate_object_reference(&tree_id, ObjectType::Tree, &objects_dir)?;

    let parent_id = parent
        .as_deref()
        .map(ObjectId::from_hex)
        .transpose()?;

    if let Some(parent_hash) = parent_id {
        validate_object_reference(&parent_hash, ObjectType::Commit, &objects_dir)?;
    }

    let commit = Commit::new(tree_id, parent_id, message)?;

    let commit_hash = write_object(&commit.to_bytes(), ObjectType::Commit, &objects_dir)?;
    Ok(commit_hash.to_string())
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
