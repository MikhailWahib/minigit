use anyhow::Result;

use crate::{porcelain::ops, repository::Repository};

pub fn init(git_mode: bool) -> Result<()> {
    let repo = Repository::init(git_mode)?;
    ops::init(&repo)?;
    println!("repo initialized at {}", repo.git_dir().display());
    Ok(())
}

pub fn add(paths: Vec<String>, repo: &Repository) -> Result<()> {
    ops::add(paths, repo)
}

pub fn commit(msg: String, repo: &Repository) -> Result<()> {
    ops::commit(msg, repo)
}

pub fn remove(paths: Vec<String>, repo: &Repository) -> Result<()> {
    ops::remove(paths, repo)
}

pub fn status(repo: &Repository) -> Result<()> {
    let (mut untracked, mut modified, mut staged) = ops::status(repo)?;
    untracked.sort();
    modified.sort();
    staged.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));

    if staged.is_empty() && modified.is_empty() && untracked.is_empty() {
        println!("nothing to commit, working tree clean");
        return Ok(());
    }

    if !staged.is_empty() {
        println!("Changes to be committed:");
        println!("  (use \"minigit remove <file>...\" to unstage)");
        println!();
        for (status, file) in &staged {
            println!("\t{status}:   {file}");
        }
        println!();
    }

    if !modified.is_empty() {
        println!("Changes not staged for commit:");
        println!("  (use \"minigit add <file>...\" to update what will be committed)");
        println!();
        for file in &modified {
            println!("\tmodified:   {file}");
        }
        println!();
    }

    if !untracked.is_empty() {
        println!("Untracked files:");
        println!("  (use \"minigit add <file>...\" to include in what will be committed)");
        println!();
        for file in &untracked {
            println!("\t{file}");
        }
        println!();
    }

    if staged.is_empty() {
        println!("no changes added to commit (use \"minigit add\" and/or \"minigit commit\")");
    }
    Ok(())
}

pub fn log(repo: &Repository) -> Result<()> {
    let commits = ops::log(repo)?;

    for (id, commit) in commits {
        println!("commit {id}");
        println!("{commit}");
    }

    Ok(())
}
