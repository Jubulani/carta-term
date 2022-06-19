use std::fs;

use anyhow::{anyhow, Context, Result};
use clap::Parser;

/// Reverse engineering and analysis of binary data in the terminal
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Carta schema file
    #[clap(short, long, value_parser)]
    schema: String,
}

#[derive(Debug)]
pub enum CartaTermError {}

fn main() -> Result<()> {
    let args = Args::parse();

    let file_data = read_file(&args.schema)?;
    let schema = carta_schema::compile_schema_file(&file_data).map_err(|err| anyhow!("{}", err))?;

    println!("Have parsed schema: {:?}", schema);
    Ok(())
}

fn read_file(schema_file: &str) -> Result<String> {
    let contents = fs::read_to_string(schema_file)
        .with_context(|| format!("Could not open file: {}", schema_file))?;
    Ok(contents)
}
