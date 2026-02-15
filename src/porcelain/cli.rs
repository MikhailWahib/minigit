use anyhow::Result;

use crate::{porcelain::ops, repository::Repository};

pub fn add(paths: Vec<String>, repo: &Repository) -> Result<()> {
    ops::add(paths, repo)?;
    Ok(())
}
