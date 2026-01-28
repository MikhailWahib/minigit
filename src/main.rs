use crate::{
    cli::{Cli, Commands},
    plumbing::commands::{cat_file, hash_object},
};
use anyhow::Result;
use clap::Parser;
use std::fs;

mod cli;
mod plumbing;

fn init() -> Result<()> {
    let path = ".minigit/objects";
    fs::create_dir_all(path)?;
    println!("repo initialized");
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.commands {
        Some(Commands::Init) => {
            init()?;
        }
        Some(Commands::HashObject { object_path, write }) => {
            hash_object(object_path, write)?;
        }
        Some(Commands::CatFile { object, typ }) => {
            cat_file(object, typ)?;
        }
        None => {
            eprintln!("No command provided. Run with --help.");
            std::process::exit(1);
        }
    }

    Ok(())
}
