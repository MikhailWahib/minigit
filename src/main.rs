use crate::cli::Cli;
use anyhow::Result;
use clap::Parser;

mod cli;
mod plumbing;

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.run()?;
    Ok(())
}
