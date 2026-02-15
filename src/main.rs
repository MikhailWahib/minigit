use crate::cli::Cli;
use anyhow::Result;
use clap::Parser;

mod cli;
mod config;
mod plumbing;
pub mod porcelain;
mod repository;
pub mod utils;

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.run()?;
    Ok(())
}
