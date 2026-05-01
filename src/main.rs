mod model;
mod renderer;
mod parser;


use clap::Parser;
use anyhow::{Context, Result};
use std::path::{ };

#[derive(Parser)]
pub struct Cli{
    path: std::path::PathBuf,

    // hides section smaller than N bytes.
    #[arg(long, value_name = "BYTES", default_value_t = 0)]
    min_size: u64,
}

#[derive(Debug)]
pub struct CustomError(String);

fn main() -> Result<()> {
    let args = Cli::parse();

    if args.path.exists(){
        anyhow::bail!("file not found: {}", args.path.display());
    }

    let map = parser::parse(&args.path)?;
}