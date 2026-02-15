use anyhow::{Context, Result};
use std::{
    env,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

pub struct Repository {
    work_tree: PathBuf,
    git_dir: PathBuf,
    git_mode: bool,
}

impl Repository {
    pub fn discover(git_mode: bool) -> Result<Self> {
        let cwd = env::current_dir()?;
        let git_dir = Self::find_git_dir(&cwd, git_mode)?;

        let work_tree = git_dir.parent().map(|p| p.to_path_buf()).unwrap_or(cwd);

        Ok(Self {
            work_tree,
            git_dir,
            git_mode,
        })
    }

    pub fn init(git_mode: bool) -> Result<Self> {
        let work_tree = env::current_dir()?;
        let git_dir_name = if git_mode { ".git" } else { ".minigit" };
        let git_dir = work_tree.join(git_dir_name);

        Ok(Self {
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

    pub fn get_ignored(&self) -> Result<Vec<PathBuf>> {
        let ignore_file_path = self.work_tree.join(self.ignore_file_name());
        let mut ignore_file =
            File::open(ignore_file_path).with_context(|| "ignore file not found")?;
        let mut ignore_content = String::new();
        ignore_file.read_to_string(&mut ignore_content)?;

        Ok(ignore_content
            .lines()
            .map(|p| self.work_tree.join(p.trim_start_matches('/')))
            .collect())
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
            if candidate.exists() {
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
