fn main() {
    if let Err(e) = findr::run(findr::Args::load()) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
