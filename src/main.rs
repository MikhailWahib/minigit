use crate::cli::Cli;
use anyhow::Result;
use clap::Parser;

mod cli;
pub mod plumbing;
pub mod repository;

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.run()?;
    Ok(())
}
