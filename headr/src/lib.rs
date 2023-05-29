use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Take};
use std::slice::Iter;
use std::{io, result};

use clap::Parser;

type Result<T> = result::Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// A Rust implementation of `head`.
pub struct Args {
    /// The list of files to read.
    #[arg(value_name = "FILE", default_value = "-")]
    files: Vec<String>,

    #[arg(
        value_name = "LINES",
        short('n'),
        long("lines"),
        conflicts_with("bytes"),
        default_value_t = 10,
        value_parser = clap::value_parser!(u64).range(1..),
    )]
    /// The number of lines to read. Mutually exclusive with --bytes.
    lines: u64,

    #[arg(
        value_name = "BYTES",
        short('c'),
        long("bytes"),
        default_value = None,
        value_parser = clap::value_parser!(u64).range(1..)
    )]
    /// The number of bytes to read. Mutually exclusive with --lines.
    bytes: Option<u64>,
}

impl Args {
    fn iter(&self) -> Iter<String> {
        self.files.iter()
    }
}

impl<'a> IntoIterator for &'a Args {
    type Item = &'a String;
    type IntoIter = Iter<'a, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

fn open(file_path: &str) -> Result<Box<dyn BufRead>> {
    let buf_read: Box<dyn BufRead> = match file_path {
        "-" => Box::new(BufReader::new(io::stdin())),
        _ => Box::new(BufReader::new(File::open(file_path)?)),
    };
    Ok(buf_read)
}

pub fn run(args: Args) -> Result<()> {
    for path in &args {
        match open(path) {
            Err(err) => eprintln!("{}: {}\n", path, err),
            Ok(reader) => head(&args, path, reader)?,
        }
    }

    Ok(())
}

fn head(args: &Args, file_path: &str, reader: Box<dyn BufRead>) -> Result<()> {
    if args.files.len() > 1 {
        print_header(file_path, file_path == args.files[0]);
    }

    if let Some(bytes) = args.bytes {
        head_bytes(reader.take(bytes))
    } else {
        head_lines(reader, args.lines)
    }
}

/// Emulate BSD `head` behavior of including a newline after each file read ONLY if:
/// - the read contents contained such a newline, OR
/// - at least one file is still to be read.
/// A straightforward implementation of this behavior is to print a leading newline for all file
/// paths but the first, allowing us to the print the file contents verbatim.
fn print_header(file_path: &str, is_first: bool) {
    println!("{}==> {} <==", if is_first { "" } else { "\n" }, file_path);
}

fn head_bytes(mut reader: Take<Box<dyn BufRead>>) -> Result<()> {
    let mut buf = vec![0u8; reader.limit() as usize];
    let bytes_read = reader.read(&mut buf)?;
    let string = String::from_utf8_lossy(&buf[..bytes_read]);
    print!("{}", string);

    Ok(())
}

fn head_lines(mut reader: Box<dyn BufRead>, lines_to_read: u64) -> Result<()> {
    let mut line = String::new();
    for _ in 0..lines_to_read {
        if reader.read_line(&mut line)? == 0 {
            break;
        }

        print!("{}", line);
        line.clear()
    }

    Ok(())
}
