use std::fs;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use crossterm::event::{read, Event, KeyCode, KeyModifiers};
use crossterm::style::Attribute;

/// Reverse engineering and analysis of binary data in the terminal
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Carta schema file
    #[clap(short, long, value_parser)]
    schema: String,

    /// Data file
    #[clap(short, long, value_parser)]
    data: String,
}

#[derive(Debug)]
pub enum CartaTermError {}

fn main() -> Result<()> {
    let args = Args::parse();
    let schema_data = read_file_as_utf8(&args.schema)?;
    let schema =
        carta_schema::compile_schema_file(&schema_data).map_err(|err| anyhow!("{}", err))?;
    print!("Schema loaded successfully\n\n");

    let file_data = read_file_as_bytes(&args.data)?;
    println!("{}", args.data);

    let nugget = carta_schema::apply_schema(&schema, &file_data);

    let hex_buffer = build_hex_buffer(&file_data);
    let ascii_buffer = build_ascii_buffer(&file_data);
    let nugget_buffer = build_nugget_buffer(nugget);

    let empty_string = String::new();
    for i in 0..std::cmp::max(
        hex_buffer.len(),
        std::cmp::max(ascii_buffer.len(), nugget_buffer.len()),
    ) {
        let hex: &str;
        let ascii: &str;
        let nugget: &str;
        if i < hex_buffer.len() {
            hex = &hex_buffer[i];
        } else {
            hex = &empty_string;
        }
        if i < ascii_buffer.len() {
            ascii = &ascii_buffer[i];
        } else {
            ascii = &empty_string;
        }
        if i < nugget_buffer.len() {
            nugget = &nugget_buffer[i];
        } else {
            nugget = &empty_string;
        }

        if i == 2 {
            println!(
                "{}{: <40}{}   {: <16}    {}",
                Attribute::Reverse,
                hex,
                Attribute::Reset,
                ascii,
                nugget
            );
        } else {
            println!("{: <40}   {: <16}    {}", hex, ascii, nugget);
        }
    }
    print_events();
    Ok(())
}

fn read_file_as_utf8(filename: &str) -> Result<String> {
    let contents = fs::read_to_string(filename)
        .with_context(|| format!("Error reading file: {}", filename))?;
    Ok(contents)
}

fn read_file_as_bytes(filename: &str) -> Result<Vec<u8>> {
    let contents =
        fs::read(filename).with_context(|| format!("Error reading file: {}", filename))?;
    Ok(contents)
}

fn build_hex_buffer(file_data: &Vec<u8>) -> Vec<String> {
    let encoded = hex::encode(file_data);
    let mut lines = Vec::new();
    let mut line = String::with_capacity(40);
    for (i, c) in encoded.chars().enumerate() {
        line.push(c);
        if (i % 4) == 3 {
            line.push(' ');
        }
        if (i % 32) == 31 {
            lines.push(line);
            line = String::with_capacity(40);
        }
    }
    if line.len() > 0 {
        lines.push(line);
    }
    lines
}

fn build_ascii_buffer(file_data: &Vec<u8>) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::with_capacity(16);
    for (i, c) in file_data.iter().enumerate() {
        if *c >= 32 && *c < 127 {
            line.push(char::from_u32(*c as u32).unwrap());
        } else {
            line.push('.');
        }
        if (i % 16) == 15 {
            lines.push(line);
            line = String::with_capacity(16);
        }
    }
    if line.len() > 0 {
        lines.push(line);
    }
    lines
}

fn build_nugget_buffer(nugget: carta_schema::Nugget) -> Vec<String> {
    let mut lines = Vec::new();
    build_nugget_buffer_recursive(nugget, &mut lines, 0);
    lines
}

fn build_nugget_buffer_recursive(
    nugget: carta_schema::Nugget,
    lines: &mut Vec<String>,
    level: usize,
) {
    let val;
    if nugget.value.is_some() {
        val = format!(": {}", nugget.value.unwrap());
    } else {
        val = String::new();
    }
    lines.push(format!(
        "{:>width$}{}",
        nugget.name,
        val,
        width = (level * 2) + nugget.name.len()
    ));
    for child in nugget.children {
        build_nugget_buffer_recursive(child, lines, level + 1);
    }
}

fn print_events() -> crossterm::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    loop {
        // `read()` blocks until an `Event` is available
        match read()? {
            Event::Key(event) => {
                if event.code == KeyCode::Char('q')
                    || event.code == KeyCode::Esc
                    || (event.code == KeyCode::Char('c')
                        && event.modifiers == KeyModifiers::CONTROL)
                    || (event.code == KeyCode::Char('d')
                        && event.modifiers == KeyModifiers::CONTROL)
                {
                    break;
                }
            }
            Event::Mouse(event) => println!("{:?}", event),
            Event::Resize(width, height) => println!("New size {}x{}", width, height),
        }
    }
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
