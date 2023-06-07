use clap::Parser;

fn main() {
    if let Err(e) = uniqr::run(uniqr::Args::parse()) {
        eprintln!("{}", e);
        std::process::exit(1)
    }
}
