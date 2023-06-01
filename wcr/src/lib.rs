use std::fmt::{Display, Write};
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::ops::AddAssign;
use std::result;
use std::slice::Iter;

use clap::Parser;

const STDIN: &str = "-";

type Result<T> = result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Parser)]
#[command(author, version, about)]
/// A Rust implementation of `wc`.
pub struct Args {
    /// The list of files to process.
    #[arg(value_name = "FILE", default_value = "-")]
    files: Vec<String>,

    /// Count the lines in the input.
    #[arg(short('l'), long("lines"))]
    lines: bool,

    /// Count the words in the input.
    #[arg(short('w'), long("words"))]
    words: bool,

    /// Count the bytes in the input.
    #[arg(short('c'), long("bytes"), conflicts_with("chars"))]
    bytes: bool,

    /// Count the Unicode characters in the input.
    #[arg(short('m'), long("chars"))]
    chars: bool,
}

impl Args {
    pub fn parse_with_defaults() -> Self {
        let mut args = Self::parse();
        if args.should_set_defaults() {
            args.set_defaults();
        }
        args
    }

    fn should_set_defaults(&self) -> bool {
        !(self.lines || self.words || self.bytes || self.chars)
    }

    fn set_defaults(&mut self) {
        self.lines = true;
        self.words = true;
        self.bytes = true;
    }

    fn iter(&self) -> Iter<'_, String> {
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

pub fn run(args: Args) -> Result<()> {
    let mut total = FileInfo::new("total");
    for file_path in &args {
        match open(file_path) {
            Err(e) => eprintln!("Failed to open {}: {}", file_path, e),
            Ok(reader) => {
                let info = FileInfo::from_reader(file_path, reader)?;
                print_file_info(&info, &args)?;
                total += info;
            }
        }
    }

    if args.files.len() > 1 {
        print_file_info(&total, &args)?;
    }

    Ok(())
}

fn open(file_path: &str) -> Result<Box<dyn BufRead>> {
    let boxed_reader: Box<dyn BufRead> = match file_path {
        STDIN => Box::new(BufReader::new(io::stdin())),
        _ => Box::new(BufReader::new(File::open(file_path)?)),
    };
    Ok(boxed_reader)
}

#[derive(Debug, PartialEq)]
pub struct FileInfo<'a> {
    name: &'a str,
    lines: usize,
    words: usize,
    bytes: usize,
    chars: usize,
}

impl<'a> FileInfo<'a> {
    fn from_reader(file_name: &'a str, mut reader: impl BufRead) -> Result<Self> {
        let mut info = Self::new(file_name);
        let mut line = String::new();
        while reader.read_line(&mut line)? != 0 {
            info.lines += 1;
            info.words += line.split_whitespace().count();
            info.bytes += line.len();
            info.chars += line.chars().count();
            line.clear();
        }
        Ok(info)
    }

    fn new(file_name: &'a str) -> Self {
        FileInfo {
            name: file_name,
            lines: 0,
            words: 0,
            bytes: 0,
            chars: 0,
        }
    }
}

impl AddAssign for FileInfo<'_> {
    fn add_assign(&mut self, other: FileInfo) {
        self.lines += other.lines;
        self.words += other.words;
        self.bytes += other.bytes;
        self.chars += other.chars;
    }
}

fn print_file_info(info: &FileInfo, args: &Args) -> Result<()> {
    let formatted = format_file_info(info, args)?;
    Ok(println!("{}", formatted))
}

fn format_file_info(info: &FileInfo, args: &Args) -> Result<impl Display> {
    let mut format = String::new();
    if args.lines {
        format_file_info_field(&mut format, info.lines)?;
    }
    if args.words {
        format_file_info_field(&mut format, info.words)?;
    }
    if args.bytes {
        format_file_info_field(&mut format, info.bytes)?;
    } else if args.chars {
        format_file_info_field(&mut format, info.chars)?;
    }
    if info.name != STDIN {
        write!(&mut format, " {}", info.name)?;
    }

    Ok(format)
}

fn format_file_info_field(mut out: impl Write, field_val: usize) -> Result<()> {
    write!(out, "{:>width$}", field_val, width = 8)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::FileInfo;

    #[test]
    fn test_count() {
        let file_name = "cursor";
        let text = "I don't want the world. I just want your half.\r\n";
        let info = FileInfo::from_reader(file_name, Cursor::new(text));
        assert!(info.is_ok());
        let expected = FileInfo {
            name: file_name,
            lines: 1,
            words: 10,
            chars: 48,
            bytes: 48,
        };
        assert_eq!(info.unwrap(), expected)
    }
}
