#![doc = include_str!("../README.md")]
#![warn(
    clippy::all,
    clippy::pedantic,
    // clippy::cargo TODO: Uncomment this once duplicate versions are resolved
)]
#![warn(missing_docs)]

use args::validate_args;
use std::{env::args, process::exit};

mod args;
mod command;
mod error;

use error::Error;

const LEGACY_ARGUMENT_LENGTH: usize = 4;

#[tokio::main]
async fn main() {
    env_logger::init();
    let args: Vec<String> = args().collect();

    let command = match validate_args(args) {
        Ok(command) => command,
        Err(err) => {
            eprintln!("{err}");
            exit(1)
        }
    };

    match command.execute().await {
        Ok(()) => exit(0),
        Err(err) => {
            eprintln!("{err}");
            exit(1)
        }
    }
}
