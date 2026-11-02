//! MOD-12: Alat baris perintah node dan wallet regtest Homeostasis Coin (`homeostasis-cli`).

#![forbid(unsafe_code)]

use std::process::ExitCode;

use homeostasis_cli::{Command, USAGE, os_random, parse_args, run, terminal_password};

const BIN_NAME: &str = "homeostasis-cli";

fn main() -> ExitCode {
    let config = match parse_args(std::env::args().skip(1)) {
        Ok(Command::Help) => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Ok(Command::Version) => {
            println!("{BIN_NAME} {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        Ok(Command::Run(config)) => config,
        Err(error) => {
            eprintln!("{BIN_NAME}: {error}");
            eprintln!();
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    match run(&config, os_random, terminal_password) {
        Ok(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{BIN_NAME}: {error}");
            ExitCode::FAILURE
        }
    }
}
