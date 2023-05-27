use clap::{Arg, ArgAction, Command};

fn main() {
    let matches = Command::new("echor")
        .version("0.1.0")
        .author("Angus Morrison")
        .about("Rust echo")
        .arg(
            Arg::new("text")
                .value_name("TEXT")
                .help("Input text")
                .required(true)
                .num_args(1..),
        )
        .arg(
            Arg::new("omit_newline")
                .short('n')
                .action(ArgAction::SetTrue)
                .help("Do not print newline"),
        )
        .get_matches();

    let text: String = matches
        .get_many("text")
        .expect("text is required")
        .map(String::as_str)
        .collect::<Vec<&str>>()
        .join(" ");

    let terminator = if matches.get_flag("omit_newline") { "" } else { "\n" };

    print!("{}{}", text, terminator);
}
