use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
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
    },
}
