use std::fs;
use std::path::Path;

use assert_cmd::Command;
use once_cell::sync::Lazy;
use predicates::prelude::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

const CRATE_NAME: &str = assert_cmd::crate_name!();

static FIXTURE_PATH: Lazy<&Path> = Lazy::new(|| Path::new("tests/expected"));

#[test]
fn fails_when_no_args_are_provided() -> TestResult {
    Command::cargo_bin(CRATE_NAME)?
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
    Ok(())
}

fn run_fixture(fixture_basename: &str, args: &[&str]) -> TestResult {
    let fixture_path = FIXTURE_PATH.join(fixture_basename);
    let expected = fs::read_to_string(fixture_path)?;
    Command::cargo_bin(CRATE_NAME)?
        .args(args)
        .assert()
        .success()
        .stdout(expected);
    Ok(())
}

#[test]
fn hello1() -> TestResult {
    run_fixture("hello1.txt", &["Hello there"])
}

#[test]
fn hello1_no_newline() -> TestResult {
    run_fixture("hello1.n.txt", &["-n", "Hello there"])
}

#[test]
fn hello2() -> TestResult {
    run_fixture("hello2.txt", &["Hello", "there"])
}

#[test]
fn hello2_no_newline() -> TestResult {
    run_fixture("hello1.n.txt", &["-n", "Hello there"])
}
