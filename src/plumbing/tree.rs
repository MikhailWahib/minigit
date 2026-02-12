pub struct Tree {
    entries: Vec<TreeEntry>,
}

impl Tree {
    pub fn from_entries(mut entries: Vec<TreeEntry>) -> Self {
        entries.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
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
