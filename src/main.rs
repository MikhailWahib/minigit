use crate::{
    cli::{Cli, Commands},
    commands::hash_object::hash_object,
};
use clap::Parser;
use std::{error::Error, fs};

mod cli;
mod commands;

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
        None => {
            eprintln!("No command provided. Run with --help.");
            std::process::exit(1);
        }
    }

    Ok(())
}
