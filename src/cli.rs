use anyhow::Result;

use clap::{Parser, Subcommand};

use crate::plumbing::cli::{
    cat_file, commit_tree, hash_object, ls_files, update_index, write_tree,
};

use crate::porcelain::cli::{add, commit, init, status};

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
    /// Add file contents to the index
    Add { paths: Vec<String> },
    /// Record changes to the repository
    Commit {
        #[arg(short)]
        message: String,
    },
    /// Show the working tree status
    Status,

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
        let Cli { git_mode, commands } = self;

        match commands {
            Commands::Init => init(git_mode),
            cmd => {
                let repo = Repository::discover(git_mode)?;
                Self::run_with_repo(cmd, &repo)
            }
        }
    }

    fn run_with_repo(cmd: Commands, repo: &Repository) -> Result<()> {
        match cmd {
            Commands::Add { paths } => {
                add(paths, repo)?;
            }
            Commands::Commit { message } => {
                commit(message, repo)?;
            }
            Commands::Status => {
                status(repo)?;
            }
            Commands::HashObject { object_path, write } => {
                let hash = hash_object(&object_path, write, repo)?;
                println!("{hash}");
            }
            Commands::CatFile { object, typ } => {
                let output = cat_file(&object, typ, repo)?;
                if typ {
                    println!("{output}");
                } else {
                    print!("{output}");
                }
            }
            Commands::LsFiles => {
                if let Some(output) = ls_files(repo)? {
                    print!("{output}");
                }
            }
            Commands::UpdateIndex { mode, object, path } => {
                update_index(&mode, &object, path, repo)?;
            }
            Commands::WriteTree => {
                let hash = write_tree(repo)?;
                println!("{hash}");
            }
            Commands::CommitTree {
                tree,
                parent,
                message,
            } => {
                let hash = commit_tree(tree, parent, message, repo)?;
                println!("{hash}");
            }
            Commands::Init => unreachable!(),
        }

        Ok(())
    }
}
