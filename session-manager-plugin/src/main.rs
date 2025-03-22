#![warn(clippy::all, clippy::restriction, clippy::pedantic, clippy::cargo)]

use args::validate_args;
use std::{env::args, process::exit};

mod args;
mod command;
mod error;

const LEGACY_ARGUMENT_LENGTH: usize = 4;

fn main() {
    env_logger::init();
    let args: Vec<String> = args().collect();

    let command = match validate_args(args) {
        Ok(command) => command,
        Err(err) => {
            eprintln!("{}", err);
            exit(1)
        }
    };

    command.execute();

    ssm_lib::session::start_session();
}
