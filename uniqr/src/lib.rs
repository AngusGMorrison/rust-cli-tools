use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::{io, result};

use clap::Parser;

type Result<T> = result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// A Rust implementation of `uniq`.
pub struct Args {
    #[arg(default_value = "-")]
    /// The file to read from.
    input_file: String,

    /// The file to write the output to.
    output_file: Option<String>,

    /// Precede each output line with the count of the number of times the line occurred in the
    /// input, followed by a single space.
    #[arg(short('c'), long("count"))]
    count: bool,
}

pub fn run(args: Args) -> Result<()> {
    let input = open_input(&args.input_file)?;
    let output = open_output(&args.output_file)?;
    uniqr(&args, input, output)
}

fn uniqr(args: &Args, mut input: Box<dyn BufRead>, mut output: Box<dyn Write>) -> Result<()> {
    let mut prev = String::new();
    let mut cur = String::new();
    let mut dup_counter = 1;

    let mut write_unique_line = |count: u32, line: &str| -> Result<()> {
        if args.count {
            write!(&mut output, "{:>4} ", count)?;
        }
        write!(&mut output, "{}", line)?;
        Ok(())
    };

    // Load the first line.
    if input.read_line(&mut prev)? == 0 {
        return Ok(()); // file is empty
    }

    // Iterate through the lines in the file, comparing each pair `(leading, trailing)` and
    // accumulating the count of adjacent, duplicate lines.
    while input.read_line(&mut cur)? != 0 {
        if lines_are_duplicates(&prev, &cur) {
            dup_counter += 1;
        } else {
            // Print the trailing line once all adjacent duplicates have been found, as indicated
            // by a new, non-matching leading line.
            write_unique_line(dup_counter, &prev)?;
            dup_counter = 1;
            std::mem::swap(&mut cur, &mut prev);
        }

        cur.clear()
    }

    write_unique_line(dup_counter, &prev)?;

    Ok(())
}

fn lines_are_duplicates(prev: &str, cur: &str) -> bool {
    match cur.chars().last() {
        Some('\n') => cur == prev,
        Some(_) => *cur == prev[..(prev.len() - 1)],
        None => false,
    }
}

fn open_input(path: &str) -> Result<Box<dyn BufRead>> {
    match path {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => {
            let file = File::open(path).map_err(|e| format_open_error(path, e))?;
            Ok(Box::new(BufReader::new(file)))
        }
    }
}

fn open_output(path: &Option<String>) -> Result<Box<dyn Write>> {
    match path {
        None => Ok(Box::new(io::stdout())),
        Some(path) => {
            let file = File::options()
                .append(true)
                .open(path)
                .map_err(|e| format_open_error(path, e))?;
            Ok(Box::new(file))
        }
    }
}

fn format_open_error(path: &str, e: io::Error) -> String {
    format!("{}: {}", path, e)
}
