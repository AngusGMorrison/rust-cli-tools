fn main() {
    if let Err(e) = wcr::run(wcr::Args::parse_with_defaults()) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
