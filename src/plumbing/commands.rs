use anyhow::{Context, Result, anyhow};
use hex;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use super::core::{format_tree, hash_content, read_object, write_object, write_tree_recursive};
use super::index::Index;
use crate::plumbing::commit::Commit;
use crate::plumbing::index_tree::IndexTree;
use crate::repository::Repository;

pub fn hash_object(object_path: &str, write: bool, repo: &Repository) -> Result<String> {
    let mut file = File::open(&object_path)
        .with_context(|| format!("Failed to open object at {}", object_path))?;

    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    if !write {
        return Ok(hash_content(&content, "blob"));
    }

    let hex_hash = write_object(&content, "blob", repo.objects_dir())?;
    Ok(hex_hash)
}

pub fn cat_file(hash: &str, typ: bool, repo: &Repository) -> Result<String> {
    let (obj_type, body) = read_object(hash, repo.objects_dir())?;

    if typ {
        return Ok(obj_type);
    }

    if obj_type == "tree" {
        let formatted = format_tree(&body)?;
        return Ok(formatted);
    }

    let body_str = std::str::from_utf8(&body).unwrap_or("<binary>");
    Ok(body_str.to_string())
}

pub fn ls_files(repo: &Repository) -> Result<Option<String>> {
    let idx_path = repo.index_path();

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
    let idx_path = repo.index_path();
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

    let mut sha1 = [0u8; 20];
    hex::decode_to_slice(object, &mut sha1)?;

    idx.add(file, sha1, mode)?;
    idx.write(idx_path)?;

    Ok(())
}

pub fn write_tree(repo: &Repository) -> Result<String> {
    let idx = Index::read(repo.index_path())?;
    let entries = idx.entries();
    let idx_dir_tree = IndexTree::from_idx_entries(entries);

    let root_hash = write_tree_recursive(&idx_dir_tree, repo.objects_dir())?;

    Ok(root_hash)
}

pub fn commit_tree(
    tree: String,
    parent: Option<String>,
    message: String,
    repo: &Repository,
) -> Result<String> {
    validate_object_reference(&tree, "tree", repo.objects_dir().as_path())?;

    if let Some(parent_hash) = &parent {
        validate_object_reference(parent_hash, "commit", repo.objects_dir().as_path())?;
    }

    let commit = Commit::new(tree, parent, message)?;

    let commit_hash = write_object(&commit.to_bytes(), "commit", repo.objects_dir())?;
    Ok(commit_hash)
}

fn validate_object_reference(object: &str, expected_type: &str, objects_dir: &Path) -> Result<()> {
    if object.len() != 40 {
        return Err(anyhow!("'{object}': is not a valid object"));
    }

    let mut sha1 = [0u8; 20];
    hex::decode_to_slice(object, &mut sha1)
        .with_context(|| format!("invalid object id '{object}': expected hex hash"))?;

    let (obj_type, _) =
        read_object(object, objects_dir).with_context(|| format!("object not found: {object}"))?;
    if obj_type != expected_type {
        return Err(anyhow!(
            "invalid object type for {object}: expected {expected_type}, got {obj_type}"
        ));
    }

    Ok(())
}
