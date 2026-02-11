use std::path::PathBuf;

pub struct Repository {
    pub root: PathBuf,
}

impl Repository {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn objects_dir(&self) -> PathBuf {
        self.root.join("objects")
    }

    pub fn index_path(&self) -> PathBuf {
        self.root.join("index")
    }
}
