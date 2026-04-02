use anyhow::Result;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::plumbing::{self};

use crate::porcelain::{self};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Use .git as a root dir instead of .minigit; Used to test on real git repos
    #[arg(long, short, global = true)]
    git_mode: bool,

    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new Minigit repository
    Init,
    /// Add file contents to the index
    Add { paths: Vec<PathBuf> },
    /// Record changes to the repository
    Commit {
        #[arg(short)]
        message: String,
    },
    /// Show the working tree status
    Status,
    /// Show commit logs
    Log,
    /// Unstage file(s) - equivalent to `git restore --staged`
    Remove { files: Vec<PathBuf> },

    /// Compute the hash of an object
    HashObject {
        #[arg(short, long)]
        /// Store the object in minigit database
        write: bool,
        /// Path for object
        object_path: PathBuf,
    },
    /// Print the contents of a file in the Minigit database
    CatFile {
        /// Instead of the content, show the object type
        #[arg(short)]
        typ: bool,
        /// The name of the object to show.
        object: String,
    },
    /// Print the index file
    LsFiles,
    /// Update the index
    UpdateIndex {
        mode: String,
        object: String,
        path: PathBuf,
    },
    /// Create a tree object from the current index
    WriteTree,
    /// Create a new commit object from the specified tree
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
            Commands::Init => porcelain::cli::init(git_mode),
            cmd => {
                let repo = Repository::discover(git_mode)?;
                Self::run_with_repo(cmd, &repo)
            }
        }
    }

    fn run_with_repo(cmd: Commands, repo: &Repository) -> Result<()> {
        match cmd {
            Commands::Add { paths } => {
                porcelain::cli::add(paths, repo)?;
            }
            Commands::Commit { message } => {
                porcelain::cli::commit(message, repo)?;
            }
            Commands::Status => {
                porcelain::cli::status(repo)?;
            }
            Commands::Log => {
                porcelain::cli::log(repo)?;
            }
            Commands::Remove { files } => {
                porcelain::cli::remove(files, repo)?;
            }
            Commands::HashObject { object_path, write } => {
                plumbing::cli::hash_object(&object_path, write, repo)?;
            }
            Commands::CatFile { object, typ } => {
                plumbing::cli::cat_file(&object, typ, repo)?;
            }
            Commands::LsFiles => {
                plumbing::cli::ls_files(repo)?;
            }
            Commands::UpdateIndex { mode, object, path } => {
                plumbing::cli::update_index(&mode, &object, path, repo)?;
            }
            Commands::WriteTree => {
                println!("{}", plumbing::cli::write_tree(repo)?);
            }
            Commands::CommitTree {
                tree,
                parent,
                message,
            } => {
                plumbing::cli::commit_tree(tree, parent, message, repo)?;
            }
            Commands::Init => unreachable!(),
        }

        Ok(())
    }
}
