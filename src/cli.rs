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
}
