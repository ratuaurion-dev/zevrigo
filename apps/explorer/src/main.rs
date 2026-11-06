//! MOD-14: Program explorer Zevrigo Coin (`zevrigo-explorer`).

#![forbid(unsafe_code)]

use std::process::ExitCode;

use zevrigo_explorer::{Command, USAGE, init_logging, parse_args, run};

const BIN_NAME: &str = "zevrigo-explorer";

fn main() -> ExitCode {
    match parse_args(std::env::args().skip(1)) {
        Ok(Command::Help) => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Ok(Command::Version) => {
            println!("{BIN_NAME} {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Ok(Command::Run(config)) => {
            init_logging();
            match run(&config) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("{BIN_NAME}: {error}");
                    ExitCode::FAILURE
                }
            }
        }
        Err(error) => {
            eprintln!("{BIN_NAME}: {error}");
            eprintln!();
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}
