use clap::Parser;

fn main() {
    if let Err(e) = headr::run(headr::Args::parse()) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
