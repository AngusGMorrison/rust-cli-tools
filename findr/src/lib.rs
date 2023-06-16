use std::ffi::OsStr;
use std::fs;
use std::os::unix::prelude::OsStrExt;

use clap::builder::PossibleValue;
use clap::ArgAction;
use clap::Parser;
use regex::bytes::Regex;
use walkdir::WalkDir;

use crate::EntryType::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
enum EntryType {
    Dir,
    File,
    Link,
}

impl clap::ValueEnum for EntryType {
    fn value_variants<'a>() -> &'a [Self] {
        &[Dir, File, Link]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Dir => PossibleValue::new("d").help("Match directories"),
            File => PossibleValue::new("f").help("Match files"),
            Link => PossibleValue::new("l").help("Match symlinks"),
        })
    }
}

#[derive(Debug, clap::Parser)]
#[command(version, about, author)]
/// findr recursively descends the directory tree for each path listed, evaluating the provided
/// expressions in terms of each file in the tree.
pub struct Args {
    #[arg(value_name = "PATH", default_value = ".")]
    paths: Vec<String>,

    /// True if the file name matches the given regex.
    #[arg(
        value_name = "NAME",
        long = "name",
        short = 'n',
        action = ArgAction::Append,
        num_args = 0..
    )]
    names: Vec<Regex>,

    #[arg(
        value_enum,
        value_name = "TYPE",
        long = "type",
        short = 't',
        action = ArgAction::Append,
        num_args = 0..,
        default_values(vec!["d", "f", "l"]),
        default_missing_values(vec!["d", "f", "l"]),
    )]
    /// True if the file is of the specified type.
    entry_types: Vec<EntryType>,
}

impl Args {
    pub fn load() -> Self {
        let mut args = Self::parse();
        args.entry_types.sort();
        args.entry_types.dedup();
        args
    }

    fn as_filter(&self) -> impl Fn(&walkdir::DirEntry) -> bool + '_ {
        |entry| {
            file_type_matches(&self.entry_types, entry.file_type())
                && file_name_matches(&self.names, entry.file_name())
        }
    }
}

#[inline]
fn file_type_matches(targets: &[EntryType], entry_type: fs::FileType) -> bool {
    targets.iter().any(|target| match target {
        Dir => entry_type.is_dir(),
        File => entry_type.is_file(),
        Link => entry_type.is_symlink(),
    })
}

#[inline]
fn file_name_matches(patterns: &[Regex], name: &OsStr) -> bool {
    patterns.is_empty()
        || patterns
            .iter()
            .any(|pattern| pattern.is_match(name.as_bytes()))
}

pub fn run(args: Args) -> Result<()> {
    for path in &args.paths {
        walk_path(&args, path)
    }

    Ok(())
}

fn walk_path(args: &Args, path: &String) {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|result| {
            if let Err(e) = &result {
                eprintln!("{}", e);
            };
            result.ok()
        })
        .filter(args.as_filter())
        .for_each(|entry| println!("{}", entry.path().display()));
}
