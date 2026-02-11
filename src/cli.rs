use anyhow::Result;
use std::fs;

use clap::{Parser, Subcommand};

use crate::plumbing::commands::{cat_file, hash_object, ls_files, update_index, write_tree};

fn get_root_dir(git_mode: bool) -> &'static str {
    if git_mode {
        return ".git";
    }

    ".minigit"
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Use .git as a root dir instead of .minigit. Used to test on real git repos
    #[arg(long, short, global = true)]
    git_mode: bool,

    #[command(subcommand)]
    commands: Commands,
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
        /// Instead of the content, show the object type
        #[arg(short)]
        typ: bool,
        /// The name of the object to show.
        #[arg()]
        object: String,
    },
    /// Prints out index file
    LsFiles,
    /// Updates the index
    UpdateIndex {
        mode: String,
        object: String,
        path: String,
    },
    /// Create a tree object from the current index
    WriteTree,
}

use crate::repository::Repository;

impl Cli {
    pub fn run(self) -> Result<()> {
        let root_dir = get_root_dir(self.git_mode);
        let repo = Repository::new(root_dir);

        match self.commands {
            Commands::Init => {
                fs::create_dir_all(repo.objects_dir())?;
                println!("repo initialized");
            }
            Commands::HashObject { object_path, write } => {
                hash_object(&object_path, write, &repo)?;
            }
            Commands::CatFile { object, typ } => {
                cat_file(&object, typ, &repo)?;
            }
            Commands::LsFiles => ls_files(&repo)?,
            Commands::UpdateIndex { mode, object, path } => {
                let mode_u32 = mode.parse::<u32>()?;
                update_index(mode_u32, &object, path, &repo)?;
            }

            Commands::WriteTree => {
                write_tree(&repo)?;
            }
        }
        Ok(())
    }
}
