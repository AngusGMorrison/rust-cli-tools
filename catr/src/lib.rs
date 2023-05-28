use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};

use cat::Cat;

pub use cat::Args;

fn open(file_path: &str) -> cat::Result<Box<dyn BufRead>> {
    match file_path {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(file_path)?))),
    }
}

pub fn run(args: Args) -> cat::Result<()> {
    for path in &args {
        match open(path) {
            Err(err) => eprintln!("Failed to open {}: {}", path, err),
            Ok(reader) => Cat::run(&args, reader)?,
        }
    }
    Ok(())
}

mod cat {
    use std::error::Error;
    use std::io::BufRead;
    use std::result;
    use std::slice::Iter;

    use clap::Parser;

    pub type Result<T> = result::Result<T, Box<dyn Error>>;

    #[derive(Debug, Parser)]
    #[command(author, version, about)]
    /// A Rust implementation of `cat`.
    pub struct Args {
        /// Input file path(s)
        #[arg(value_name = "FILE", default_value = "-")]
        files: Vec<String>,

        /// Number the output lines, starting at 1
        #[arg(short('n'), long("number"), conflicts_with("number_nonblank_lines"))]
        number_lines: bool,

        /// Number the non-blank output lines, starting at 1
        #[arg(short('b'), long("number-nonblank"))]
        number_nonblank_lines: bool,

        /// Squeeze multiple adjacent empty lines, causing the output to be single-spaced
        #[arg(short('s'), long("squeeze"))]
        squeeze: bool,
    }

    impl Args {
        // Return an immutable iterator over the list of files passed as arguments.
        fn iter(&self) -> Iter<'_, String> {
            self.files.iter()
        }
    }

    // Implicitly convert references to `Args` to immutable iterators, e.g. in loops.
    impl<'a> IntoIterator for &'a Args {
        // The type of item yielded by the iterator.
        type Item = &'a String;
        // The concrete type of the resulting iterator.
        // `Iter` is an immutable iterator over a slice of strings, yielding `&String`.
        type IntoIter = Iter<'a, String>;

        // Return an instance of the concrete iterator type.
        fn into_iter(self) -> Self::IntoIter {
            self.iter()
        }
    }

    pub struct Cat<'a> {
        args: &'a Args,
        reader: Box<dyn BufRead>,
        line_counter: u32,
        cur_line: String,
        prev_line_empty: bool,
    }

    impl Cat<'_> {
        pub fn run(args: &Args, reader: Box<dyn BufRead>) -> Result<()> {
            Cat::new(args, reader).cat()
        }

        fn new(args: &Args, reader: Box<dyn BufRead>) -> Cat<'_> {
            Cat {
                args,
                reader,
                line_counter: 1,
                prev_line_empty: false,
                cur_line: String::new(),
            }
        }

        fn cat(&mut self) -> Result<()> {
            // `BufRead.lines` omits trailing newlines, which makes it impossible to emulate the behavior of
            // `cat`, since we have no way of determining whether the file ends with a newline.
            // `BufRead.read_line` preserves newlines.
            while self.reader.read_line(&mut self.cur_line)? != 0 {
                self.cat_line();
                self.cur_line.clear();
            }

            Ok(())
        }

        fn cat_line(&mut self) {
            let cur_line_empty = self.cur_line.trim().is_empty();

            // Squeeze
            if self.args.squeeze && self.prev_line_empty && cur_line_empty {
                return;
            }

            // Number
            self.number_line(cur_line_empty);

            print!("{}", self.cur_line);

            self.prev_line_empty = cur_line_empty;
        }

        fn number_line(&mut self, cur_line_empty: bool) {
            if self.args.number_lines || (self.args.number_nonblank_lines && !cur_line_empty) {
                print!("{:6}\t", self.line_counter);
                self.line_counter += 1;
            }
        }
    }
}
