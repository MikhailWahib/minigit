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

use crate::plumbing::{
    index_tree::IndexTree,
    reader::Reader,
    tree::{Tree, TreeEntry},
};

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

pub fn write_tree_recursive(node: &IndexTree, objects_path: impl AsRef<Path>) -> Result<String> {
    match node {
        IndexTree::Blob { sha1, .. } => Ok(hex::encode(sha1)),
        IndexTree::Tree { children } => {
            let mut entries = Vec::new();

            for (name, child_node) in children {
                match child_node {
                    IndexTree::Blob { mode, sha1, .. } => {
                        entries.push(TreeEntry::blob(name.clone(), *mode, *sha1));
                    }
                    IndexTree::Tree { .. } => {
                        let subtree_hash = write_tree_recursive(child_node, objects_path.as_ref())?;
                        let mut sha1 = [0u8; 20];
                        hex::decode_to_slice(&subtree_hash, &mut sha1)?;

                        entries.push(TreeEntry::tree(name.clone(), sha1));
                    }
                }
            }

            let tree = Tree::from_entries(entries);
            write_object(&tree.to_bytes(), "tree", objects_path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{format_tree, read_object, write_object, write_tree_recursive};
    use crate::plumbing::index_tree::IndexTree;
    use tempfile::tempdir;

    #[test]
    fn writes_and_reads_object_roundtrip() {
        let dir = tempdir().expect("tempdir");
        let content = b"hello minigit";

        let hash = write_object(content, "blob", dir.path()).expect("write_object");
        let (obj_type, body) = read_object(&hash, dir.path()).expect("read_object");

        assert_eq!(obj_type, "blob");
        assert_eq!(body, content);
        assert_eq!(hash.len(), 40);
    }

    #[test]
    fn formats_tree_entries() {
        let blob_sha = [0xAA; 20];
        let tree_sha = [0xBB; 20];
        let data = [
            b"100644 file.txt\0".to_vec(),
            blob_sha.to_vec(),
            b"40000 dir\0".to_vec(),
            tree_sha.to_vec(),
        ]
        .concat();

        let formatted = format_tree(&data).expect("format_tree");

        assert!(formatted.contains("100644 blob "));
        assert!(formatted.contains("\tfile.txt"));
        assert!(formatted.contains("040000 tree "));
        assert!(formatted.contains("\tdir"));
    }

    #[test]
    fn rejects_malformed_tree_entry() {
        let err = format_tree(b"100644-no-space\0").expect_err("invalid tree should fail");
        assert!(format!("{err}").contains("Invalid tree entry format"));
    }

    #[test]
    fn writes_nested_tree_objects_recursively() {
        let dir = tempdir().expect("tempdir");
        let subtree = IndexTree::Tree {
            children: std::collections::BTreeMap::from([(
                "inner.txt".to_string(),
                IndexTree::Blob {
                    mode: 0o100644,
                    sha1: [0x22; 20],
                },
            )]),
        };
        let expected_sub_hash =
            write_tree_recursive(&subtree, dir.path()).expect("write subtree recursively");

        let root = IndexTree::Tree {
            children: std::collections::BTreeMap::from([
                (
                    "file.txt".to_string(),
                    IndexTree::Blob {
                        mode: 0o100644,
                        sha1: [0x11; 20],
                    },
                ),
                (
                    "sub".to_string(),
                    subtree,
                ),
            ]),
        };

        let root_hash = write_tree_recursive(&root, dir.path()).expect("write_tree_recursive");
        let (obj_type, body) = read_object(&root_hash, dir.path()).expect("read root tree");
        let pretty = format_tree(&body).expect("format root tree");
        let expected = format!(
            "100644 blob {}\tfile.txt\n040000 tree {}\tsub\n",
            hex::encode([0x11; 20]),
            expected_sub_hash
        );

        assert_eq!(obj_type, "tree");
        assert_eq!(pretty, expected);
    }
}
