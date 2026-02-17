use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::Path;

use super::core::{format_tree, hash_content, read_object, write_object, write_tree_recursive};
use super::index::Index;
use super::object::{ObjectId, ObjectType};
use super::reader::Reader;
use crate::plumbing::commit::Commit;
use crate::plumbing::index_tree::IndexTree;
use crate::repository::Repository;

pub struct TreeEntryData {
    pub name: String,
    pub object_id: ObjectId,
    pub object_type: ObjectType,
}

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

pub fn read_commit(object_id: ObjectId, repo: &Repository) -> Result<Commit> {
    let objects_dir = repo.git_dir().join("objects");
    let (obj_type, body) = read_object(&object_id, &objects_dir)?;
    if obj_type != ObjectType::Commit {
        return Err(anyhow!(
            "invalid object type for {object_id}: expected commit, got {obj_type}"
        ));
    }

    Commit::from_bytes(&body)
}

pub fn read_tree(object_id: ObjectId, repo: &Repository) -> Result<Vec<TreeEntryData>> {
    let objects_dir = repo.git_dir().join("objects");
    let (obj_type, body) = read_object(&object_id, &objects_dir)?;
    if obj_type != ObjectType::Tree {
        return Err(anyhow!(
            "invalid object type for {object_id}: expected tree, got {obj_type}"
        ));
    }

    let mut entries = Vec::new();
    let mut reader = Reader::new(&body);
    while !reader.is_eof() {
        let mode_and_name = std::str::from_utf8(reader.read_until_nul()?)?;
        let (mode_str, name) = mode_and_name
            .split_once(' ')
            .ok_or_else(|| anyhow!("Invalid tree entry format"))?;
        let mode = mode_str.parse::<u32>()?;

        let object_id = ObjectId::from_bytes(reader.read_exact(20)?.try_into()?);
        let object_type = if mode == 0o040000 {
            ObjectType::Tree
        } else {
            ObjectType::Blob
        };

        entries.push(TreeEntryData {
            name: name.to_string(),
            object_id,
            object_type,
        });
    }

    Ok(entries)
}

pub fn ls_files(repo: &Repository) -> Result<Option<String>> {
    let idx_path = repo.git_dir().join("index");

    Ok(Index::read_optional(idx_path)?.map(|idx| idx.to_string()))
}

pub fn update_index(mode: u32, object: ObjectId, file: String, repo: &Repository) -> Result<()> {
    let idx_path = repo.git_dir().join("index");
    let mut idx = Index::read_or_new(&idx_path)?;

    idx.add(file, object.as_bytes(), mode)?;
    idx.write(idx_path)?;

    Ok(())
}

pub fn remove_from_index(file: String, repo: &Repository) -> Result<()> {
    let idx_path = repo.git_dir().join("index");
    let mut idx = Index::read_or_new(&idx_path)?;

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
