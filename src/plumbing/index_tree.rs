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
