use std::{env, process};
use mygrep::Config;
use text_colorizer::*;

fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("{} Problem parsing arguments: {}", "Error: ".red().bold(), err);
        process::exit(1);
    });

    if let Err(err) = mygrep::run(config) {
        eprintln!("{} Application error: {}", "Error: ".red().bold(), err);
        process::exit(1);
    }
}
