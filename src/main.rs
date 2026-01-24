use std::fs;

use crate::cli::{Cli, Commands};
use clap::Parser;
mod cli;

fn init() {
    let paths = [".minigit/objects"];

    paths.iter().for_each(|p| fs::create_dir_all(p).unwrap());

    println!("repo initialized");
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init) => init(),
        Some(Commands::HashObject { write }) => println!("Hashing object with write = {}", write),
        None => {
            eprintln!("No command provided. Run with --help.");
            std::process::exit(1);
        }
    }
}
