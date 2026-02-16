use anyhow::Result;

use crate::{porcelain::ops, repository::Repository};

pub fn init(git_mode: bool) -> Result<()> {
    let repo = Repository::init(git_mode)?;
    ops::init(&repo)?;
    println!("repo initialized at {}", repo.git_dir().to_str().unwrap());
    Ok(())
}

pub fn add(paths: Vec<String>, repo: &Repository) -> Result<()> {
    ops::add(paths, repo)?;
    Ok(())
}
