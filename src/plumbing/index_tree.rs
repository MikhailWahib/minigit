use std::collections::BTreeMap;

use crate::plumbing::index::IndexEntry;

#[derive(Debug)]
pub enum IndexTree {
    Blob { mode: u32, sha1: [u8; 20] },
    Tree { children: BTreeMap<String, IndexTree> },
}

impl IndexTree {
    pub fn from_idx_entries(idx_entries: Vec<&IndexEntry>) -> Self {
        let mut root = IndexTree::Tree {
            children: BTreeMap::new(),
        };

        for entry in idx_entries {
            Self::insert_entry(&mut root, entry);
        }

        root
    }

    fn insert_entry(root: &mut IndexTree, entry: &IndexEntry) {
        let parts: Vec<&str> = entry.name.split('/').collect();
        let mut cur = root;

        for (i, part) in parts.iter().enumerate() {
            let is_leaf = i == parts.len() - 1;

            cur = match cur {
                IndexTree::Tree { children } => {
                    if is_leaf {
                        children.insert(
                            part.to_string(),
                            IndexTree::Blob {
                                mode: entry.mode,
                                sha1: entry.sha1,
                            },
                        );
                        return;
                    } else {
                        children
                            .entry(part.to_string())
                            .or_insert_with(|| IndexTree::Tree {
                                children: BTreeMap::new(),
                            })
                    }
                }
                _ => unreachable!(),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IndexTree;
    use crate::plumbing::index::Index;
    use std::fs;
    use tempfile::tempdir_in;

    #[test]
    fn builds_nested_structure_from_index_entries() {
        let tmp = tempdir_in(".").expect("tempdir");
        let base = tmp.path().file_name().expect("tmp dir name").to_string_lossy();

        let root_file = format!("{base}/root.txt");
        let nested_file = format!("{base}/dir/nested.txt");
        fs::create_dir_all(format!("{base}/dir")).expect("mkdir");
        fs::write(&root_file, "root").expect("write root");
        fs::write(&nested_file, "nested").expect("write nested");

        let mut index = Index::new();
        index
            .add(root_file.clone(), [0x10; 20], 0o100644)
            .expect("add root");
        index
            .add(nested_file, [0x20; 20], 0o100644)
            .expect("add nested");

        let tree = IndexTree::from_idx_entries(index.entries());

        match tree {
            IndexTree::Tree { children } => {
                let top = children.get(base.as_ref()).expect("top-level directory");
                match top {
                    IndexTree::Tree { children } => {
                        assert!(children.contains_key("root.txt"));
                        assert!(children.contains_key("dir"));
                    }
                    _ => panic!("top entry should be a tree"),
                }
            }
            _ => panic!("root should be a tree"),
        }
    }
}
