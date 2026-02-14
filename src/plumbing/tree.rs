pub struct Tree {
    entries: Vec<TreeEntry>,
}

impl Tree {
    pub fn from_entries(mut entries: Vec<TreeEntry>) -> Self {
        entries.sort_by_key(|a| a.sort_key());
        Self { entries }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for entry in &self.entries {
            bytes.extend_from_slice(format!("{:o} {}\0", entry.mode, entry.name).as_bytes());
            bytes.extend_from_slice(&entry.sha1);
        }
        bytes
    }
}

pub struct TreeEntry {
    mode: u32,
    name: String,
    sha1: [u8; 20],
}

impl TreeEntry {
    pub fn blob(name: impl Into<String>, mode: u32, sha1: [u8; 20]) -> Self {
        Self {
            mode,
            name: name.into(),
            sha1,
        }
    }

    pub fn tree(name: impl Into<String>, sha1: [u8; 20]) -> Self {
        Self {
            mode: 0o040000,
            name: name.into(),
            sha1,
        }
    }

    fn sort_key(&self) -> String {
        if self.mode == 0o040000 {
            format!("{}/", self.name)
        } else {
            self.name.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Tree, TreeEntry};

    #[test]
    fn serializes_entries_in_git_sort_order() {
        let tree_sha = [0x11; 20];
        let blob_sha_a = [0x22; 20];
        let blob_sha_z = [0x33; 20];

        let tree = Tree::from_entries(vec![
            TreeEntry::blob("z.txt", 0o100644, blob_sha_z),
            TreeEntry::blob("a.txt", 0o100644, blob_sha_a),
            TreeEntry::tree("dir", tree_sha),
        ]);

        let bytes = tree.to_bytes();
        let expected = [
            format!("{:o} {}\0", 0o100644, "a.txt").into_bytes(),
            blob_sha_a.to_vec(),
            format!("{:o} {}\0", 0o040000, "dir").into_bytes(),
            tree_sha.to_vec(),
            format!("{:o} {}\0", 0o100644, "z.txt").into_bytes(),
            blob_sha_z.to_vec(),
        ]
        .concat();

        assert_eq!(bytes, expected);
    }
}
