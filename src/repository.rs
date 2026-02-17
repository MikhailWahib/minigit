use anyhow::Result;
use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
};

pub struct Repository {
    cwd: PathBuf,
    work_tree: PathBuf,
    git_dir: PathBuf,
    git_mode: bool,
}

impl Repository {
    pub fn discover(git_mode: bool) -> Result<Self> {
        let cwd = env::current_dir()?;
        let git_dir = Self::find_git_dir(&cwd, git_mode)?;

        let work_tree = git_dir
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| cwd.clone());

        Ok(Self {
            cwd,
            work_tree,
            git_dir,
            git_mode,
        })
    }

    pub fn init(git_mode: bool) -> Result<Self> {
        let cwd = env::current_dir()?;
        let work_tree = cwd.clone();
        let git_dir_name = if git_mode { ".git" } else { ".minigit" };
        let git_dir = work_tree.join(git_dir_name);

        Ok(Self {
            cwd,
            work_tree,
            git_dir,
            git_mode,
        })
    }

    pub fn work_tree(&self) -> &Path {
        &self.work_tree
    }

    pub fn git_dir(&self) -> &Path {
        &self.git_dir
    }

    pub fn resolve_from_cwd(&self, path: impl AsRef<Path>) -> PathBuf {
        let path = path.as_ref();
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.cwd.join(path)
        }
    }

    pub fn objects_dir(&self) -> PathBuf {
        self.git_dir.join("objects")
    }

    pub fn index_path(&self) -> PathBuf {
        self.git_dir.join("index")
    }

    pub fn get_ignored(&self) -> Result<Vec<PathBuf>> {
        let ignore_file_path = self.work_tree.join(self.ignore_file_name());
        let ignore_content = fs::read_to_string(ignore_file_path).unwrap_or_default();
        let mut ignored = BTreeSet::new();

        for raw in ignore_content.lines().chain([".minigit", ".git"]) {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            ignored.insert(self.work_tree.join(line.trim_start_matches('/')));
        }

        Ok(ignored.into_iter().collect())
    }

    fn ignore_file_name(&self) -> &str {
        if self.git_mode {
            ".gitignore"
        } else {
            ".minigitignore"
        }
    }

    fn find_git_dir(start: &Path, git_mode: bool) -> Result<PathBuf> {
        let mut current = start;
        let git_dir = if git_mode { ".git" } else { ".minigit" };

        loop {
            let candidate = current.join(git_dir);
            if candidate.is_dir() {
                return Ok(candidate);
            }

            match current.parent() {
                Some(parent) => current = parent,
                None => {
                    return Err(anyhow::anyhow!(
                        "not a {} repository (or any parent up to mount point /)",
                        &git_dir[1..]
                    ));
                }
            }
        }
    }
}
