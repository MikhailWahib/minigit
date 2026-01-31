use anyhow::Result;
use std::{fs, io};

use crate::plumbing::{
    commands::{cat_file, hash_object, ls_files},
    index::Index,
};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new minigit repo
    Init,
    /// Hash an object
    HashObject {
        #[arg(short, long)]
        /// Store the object in minigit database
        write: bool,
        /// Path for object
        #[arg()]
        object_path: String,
    },
    CatFile {
        #[arg(short)]
        /// Instead of the content, show the object type
        typ: bool,
        /// The name of the object to show.
        #[arg()]
        object: String,
    },
    LsFiles,
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        match &self.commands {
            Some(Commands::Init) => {
                // TODO: accept optional path arg from cli
                // to choose from .minigit and .git
                fs::create_dir_all(".git/objects")?;
                println!("repo initialized");
            }
            Some(Commands::HashObject { object_path, write }) => {
                hash_object(object_path, write)?;
            }
            Some(Commands::CatFile { object, typ }) => {
                cat_file(object, typ)?;
            }
            Some(Commands::LsFiles) => ls_files(".git/index")?,
            None => {
                eprintln!("No command provided. Run with --help.");
                std::process::exit(1);
            }
        }
        Ok(())
    }
}
