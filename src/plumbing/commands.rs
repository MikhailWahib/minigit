use anyhow::{Context, Result};
use hex;
use std::fs::File;
use std::io::{self, Read};

use super::core::{format_tree, hash_content, read_object, write_object, write_tree_recursive};
use super::index::Index;
use crate::plumbing::index_tree::IndexTree;
use crate::repository::Repository;

pub fn hash_object(object_path: &str, write: bool, repo: &Repository) -> Result<()> {
    let mut file = File::open(&object_path)
        .with_context(|| format!("Failed to open object at {}", object_path))?;

    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    if !write {
        let hex_hash = hash_content(&content, "blob");
        println!("{hex_hash}");
        return Ok(());
    }

    let hex_hash = write_object(&content, "blob", repo.objects_dir())?;
    println!("{hex_hash}");

    Ok(())
}

pub fn cat_file(hash: &str, typ: bool, repo: &Repository) -> Result<String> {
    let (obj_type, body) = read_object(hash, repo.objects_dir())?;

    if typ {
        println!("{obj_type}");
        return Ok(obj_type);
    }

    if obj_type == "tree" {
        let formatted = format_tree(&body)?;
        print!("{}", formatted);
        return Ok(formatted);
    }

    let body_str = std::str::from_utf8(&body).unwrap_or("<binary>");
    println!("{body_str}");
    Ok(body_str.to_string())
}

pub fn ls_files(repo: &Repository) -> Result<()> {
    let idx_path = repo.index_path();

    match Index::read(idx_path) {
        Ok(idx) => println!("{}", idx),
        Err(e)
            if e.downcast_ref::<io::Error>()
                .is_some_and(|e| e.kind() == io::ErrorKind::NotFound) =>
        {
            // ignore missing index
        }
        Err(e) => return Err(e),
    }

    Ok(())
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

    println!("{}", root_hash);
    Ok(root_hash)
}
