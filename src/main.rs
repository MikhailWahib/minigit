use crate::{
    cli::{Cli, Commands},
    plumbing::commands::{cat_file, hash_object},
};
use clap::Parser;
use std::{error::Error, fs};

mod cli;
mod plumbing;

fn init() {
    let paths = [".minigit/objects"];

    paths.iter().for_each(|p| fs::create_dir_all(p).unwrap());

    println!("repo initialized");
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.commands {
        Some(Commands::Init) => init(),
        Some(Commands::HashObject { object_path, write }) => {
            hash_object(object_path.clone(), write.clone())?;
        }
        Some(Commands::CatFile { object, typ }) => {
            cat_file(object.clone(), typ.clone())?;
        }
        None => {
            eprintln!("No command provided. Run with --help.");
            std::process::exit(1);
        }
    }

    Ok(())
}
