use anyhow::{Context, Result, anyhow};
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use hex;
use sha1::{Digest, Sha1};
use std::{
    fs::{File, create_dir_all},
    io::{Read, Write},
    path::Path,
};

use crate::plumbing::{dir_tree::DirTree, reader::Reader};

/// Write a Git object (blob, tree, commit, etc.) to the object database
pub fn write_object(
    content: &[u8],
    obj_type: &str,
    objects_path: impl AsRef<Path>,
) -> Result<String> {
    let header = format!("{} {}\0", obj_type, content.len());

    let mut hasher = Sha1::new();
    hasher.update(header.as_bytes());
    hasher.update(content);
    let hash = hasher.finalize();
    let hex_hash = hex::encode(hash);

    let (dir, file_name) = hex_hash.split_at(2);
    let obj_dir = objects_path.as_ref().join(dir);
    let obj_path = obj_dir.join(file_name);

    if !obj_path.exists() {
        create_dir_all(&obj_dir)?;

        let mut object = Vec::new();
        object.extend_from_slice(header.as_bytes());
        object.extend_from_slice(content);

        let file = File::create(&obj_path)?;
        let mut encoder = ZlibEncoder::new(file, Compression::default());
        encoder.write_all(&object)?;
        encoder.finish()?;
    }

    Ok(hex_hash)
}

/// Read and decompress a Git object from the object database
pub fn read_object(hash: &str, objects_path: impl AsRef<Path>) -> Result<(String, Vec<u8>)> {
    let (dir, file_name) = hash.split_at(2);

    let file_dir = objects_path.as_ref().join(dir);
    let file_path = file_dir.join(file_name);

    let file = File::open(&file_path)
        .with_context(|| format!("Could not find object with hash {}", hash))?;

    let mut decoder = ZlibDecoder::new(file);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;

    let pos = decompressed
        .iter()
        .position(|&b| b == 0)
        .ok_or_else(|| anyhow!("Invalid object data: no null separator found"))?;

    let header = &decompressed[..pos];
    let body = &decompressed[pos + 1..];

    let header_str = std::str::from_utf8(header)?;
    let obj_type = header_str
        .split(' ')
        .next()
        .ok_or_else(|| anyhow!("Malformed object header"))?
        .to_string();

    Ok((obj_type, body.to_vec()))
}

/// Hash content without writing to object database
pub fn hash_content(content: &[u8], obj_type: &str) -> String {
    let header = format!("{} {}\0", obj_type, content.len());
    let mut hasher = Sha1::new();
    hasher.update(header.as_bytes());
    hasher.update(content);
    let hash = hasher.finalize();
    hex::encode(hash)
}

/// Format tree data for pretty printing
pub fn format_tree(data: &[u8]) -> Result<String> {
    let mut output = String::new();
    let mut reader = Reader {
        buf: data,
        offset: 0,
    };

    while reader.offset < reader.buf.len() {
        // read until \0 byte to get "mode name"
        let start = reader.offset;
        let null_pos = reader.buf[start..]
            .iter()
            .position(|&b| b == 0)
            .ok_or_else(|| anyhow!("Malformed tree entry"))?;

        let mode_name = std::str::from_utf8(&reader.buf[start..start + null_pos])?;
        let parts: Vec<&str> = mode_name.split(' ').collect();

        if parts.len() != 2 {
            return Err(anyhow!("Invalid tree entry format"));
        }

        let mode: u32 = parts[0].parse()?;
        let name = parts[1];

        // skip \0
        reader.skip(null_pos + 1)?;

        let sha1 = reader.read_exact(20)?;
        let sha1_hex = hex::encode(sha1);

        let obj_type = if mode == 40000 { "tree" } else { "blob" };

        output.push_str(&format!(
            "{:06} {} {}\t{}\n",
            mode, obj_type, sha1_hex, name
        ));
    }

    Ok(output)
}

pub fn write_tree_recursive(node: &DirTree, objects_path: impl AsRef<Path>) -> Result<String> {
    match node {
        DirTree::Blob { sha1, .. } => Ok(hex::encode(sha1)),
        DirTree::Tree { children } => {
            let mut entries = Vec::new();

            for (name, child_node) in children {
                match child_node {
                    DirTree::Blob { mode, sha1, .. } => {
                        entries.push(TreeEntryData {
                            mode: *mode,
                            name: name.clone(),
                            sha1: *sha1,
                        });
                    }
                    DirTree::Tree { .. } => {
                        let subtree_hash = write_tree_recursive(child_node, objects_path.as_ref())?;
                        let mut sha1 = [0u8; 20];
                        hex::decode_to_slice(&subtree_hash, &mut sha1)?;

                        entries.push(TreeEntryData {
                            mode: 0o040000,
                            name: name.clone(),
                            sha1,
                        });
                    }
                }
            }

            // git sorts tree entries with special rules:
            // directories are sorted as if they have a trailing '/'
            // this means "foo.rs" comes before "foo/"
            // so if the entry is a dir (tree), wee add a trailing '/' to sort properly
            entries.sort_by(|a, b| {
                let a_name = if a.mode == 0o040000 {
                    format!("{}/", a.name)
                } else {
                    a.name.clone()
                };
                let b_name = if b.mode == 0o040000 {
                    format!("{}/", b.name)
                } else {
                    b.name.clone()
                };
                a_name.cmp(&b_name)
            });

            let mut tree_content = Vec::new();
            for entry in &entries {
                tree_content
                    .extend_from_slice(format!("{:o} {}\0", entry.mode, entry.name).as_bytes());
                tree_content.extend_from_slice(&entry.sha1);
            }

            write_object(&tree_content, "tree", objects_path)
        }
    }
}

struct TreeEntryData {
    mode: u32,
    name: String,
    sha1: [u8; 20],
}
