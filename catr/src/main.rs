use clap::Parser;

fn main() {
    if let Err(e) = catr::run(catr::Args::parse()) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
