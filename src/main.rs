mod app;
mod config;
mod credentials;
mod editing;
mod engine;
mod siwc;

use engine::{edit_selection, polish};
use std::io::{self, Read};

fn main() {
    let tweet = std::env::args().any(|a| a == "--tweet");
    let selection = std::env::args().any(|a| a == "--polish-selection");
    if selection || tweet || std::env::args().any(|a| a == "--polish") {
        let mut input = String::new();
        let result = io::stdin()
            .read_to_string(&mut input)
            .map_err(|e| e.to_string())
            .and_then(|_| {
                if selection {
                    edit_selection(&input)
                } else {
                    polish(&input, tweet)
                }
            });
        match result {
            Ok(output) => print!("{output}"),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
    } else {
        app::run();
    }
}
