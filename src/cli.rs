use anyhow::Result;
use std::fs;

use clap::{Parser, Subcommand};

use crate::plumbing::commands::{
    cat_file, commit_tree, hash_object, ls_files, update_index, write_tree,
};

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
        object_path: String,
    },
    /// Prints out the content of a given file in minigit database
    CatFile {
        /// Instead of the content, show the object type
        #[arg(short)]
        typ: bool,
        /// The name of the object to show.
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
    /// Creates a tree object from the current index
    WriteTree,
    /// Creates a new commit object based on the provided tree object
    CommitTree {
        tree: String,
        #[arg(short)]
        parent: Option<String>,
        #[arg(short)]
        message: String,
    },
}

use crate::repository::Repository;

impl Cli {
    pub fn run(self) -> Result<()> {
        match self.commands {
            Commands::Init => {
                let repo = Repository::init(self.git_mode)?;
                fs::create_dir_all(repo.objects_dir())?;
                println!("repo initialized");
            }
            Commands::HashObject { object_path, write } => {
                let repo = Repository::discover(self.git_mode)?;
                hash_object(&object_path, write, &repo)?;
            }
            Commands::CatFile { object, typ } => {
                let repo = Repository::discover(self.git_mode)?;
                cat_file(&object, typ, &repo)?;
            }
            Commands::LsFiles => {
                let repo = Repository::discover(self.git_mode)?;
                ls_files(&repo)?;
            }
            Commands::UpdateIndex { mode, object, path } => {
                let repo = Repository::discover(self.git_mode)?;
                let mode_u32 = mode.parse::<u32>()?;
                update_index(mode_u32, &object, path, &repo)?;
            }

            Commands::WriteTree => {
                let repo = Repository::discover(self.git_mode)?;
                write_tree(&repo)?;
            }
            Commands::CommitTree {
                tree,
                parent,
                message,
            } => {
                let repo = Repository::discover(self.git_mode)?;
                commit_tree(tree, parent, message, &repo)?;
            }
        }
        Ok(())
    }
}
