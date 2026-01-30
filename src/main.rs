use crate::{
    cli::{Cli, Commands},
    plumbing::{
        commands::{cat_file, hash_object},
        index::Index,
    },
};
use anyhow::Result;
use clap::Parser;
use std::fs;

mod cli;
mod plumbing;

fn init(path: &str) -> Result<()> {
    fs::create_dir_all(path)?;
    println!("repo initialized");
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.commands {
        Some(Commands::Init) => {
            // TODO: accept optional path arg from cli
            // to choose from .minigit and .git
            init(".minigit/objects")?;
        }
        Some(Commands::HashObject { object_path, write }) => {
            hash_object(object_path, write)?;
        }
        Some(Commands::CatFile { object, typ }) => {
            cat_file(object, typ)?;
        }
        Some(Commands::LsFiles) => {
            let mut idx = Index::new();
            idx.init(".git/index")?;
            println!("{}", idx);
        }
        None => {
            eprintln!("No command provided. Run with --help.");
            std::process::exit(1);
        }
    }

    Ok(())
}
