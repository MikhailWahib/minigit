use anyhow::Result;
use std::{
    env,
    path::{Path, PathBuf},
};

pub struct Repository {
    pub work_tree: PathBuf,
    pub git_dir: PathBuf,
    git_mode: bool,
}

impl Repository {
    pub fn new(git_mode: bool) -> Result<Self> {
        let cwd = env::current_dir()?;
        let git_dir = Self::find_git_dir(&cwd, git_mode)?;

        let work_tree = git_dir.parent().map(|p| p.to_path_buf()).unwrap_or(cwd);

        Ok(Self {
            work_tree,
            git_dir,
            git_mode,
        })
    }

    pub fn git_dir(&self) -> PathBuf {
        self.git_dir.clone()
    }

    pub fn objects_dir(&self) -> PathBuf {
        self.git_dir.join("objects")
    }

    pub fn index_path(&self) -> PathBuf {
        self.git_dir.join("index")
    }

    pub fn ignore_file_path(&self) -> PathBuf {
        let name = if self.git_mode {
            ".gitignore"
        } else {
            ".minigitignore"
        };

        self.work_tree.join(name)
    }

    fn find_git_dir(start: &Path, git_mode: bool) -> Result<PathBuf> {
        let mut current = start;
        let git_dir = if git_mode { ".git" } else { ".minigit" };

        loop {
            let candidate = current.join(git_dir);
            if candidate.exists() {
                return Ok(candidate);
            }

            match current.parent() {
                Some(parent) => current = parent,
                None => {
                    return Err(anyhow::anyhow!(
                        "not a {} repository (or any parent up to mount point /)",
                        git_dir[1..].to_string()
                    ));
                }
            }
        }
    }
}
