use anyhow::Result;
use std::{fs, path::Path};

use crate::plumbing::commands::{cat_file, hash_object, ls_files, update_index};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Commands,
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
    /// Prints out the content of a given file in minigit database
    CatFile {
        #[arg(short)]
        /// Instead of the content, show the object type
        typ: bool,
        /// The name of the object to show.
        #[arg()]
        object: String,
    },
    /// Prints out index file
    LsFiles,
    /// Updates the index
    UpdateIndex {
        #[arg(long, num_args = 3, value_names = ["MODE", "OBJECT", "FILE"])]
        cacheinfo: Vec<String>,
    },
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        match &self.commands {
            Commands::Init => {
                // TODO: accept optional path arg from cli
                // to choose from .minigit and .git
                fs::create_dir_all(".minigit/objects")?;
                println!("repo initialized");
            }
            Commands::HashObject { object_path, write } => {
                hash_object(object_path, write)?;
            }
            Commands::CatFile { object, typ } => {
                cat_file(object, typ)?;
            }
            Commands::LsFiles => ls_files(Path::new(".minigit/index"))?,
            Commands::UpdateIndex { cacheinfo } => {
                let [mode, sha, path] = &cacheinfo[..] else {
                    unreachable!("Clap ensures 3 args")
                };

                update_index(mode, sha, path)?;
            }
        }
        Ok(())
    }
}
