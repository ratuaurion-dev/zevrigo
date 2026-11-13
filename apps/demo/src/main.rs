//! MOD-18: Program layanan jaringan demo regtest Zevrigo Coin (`zevrigo-demo`, DEC-34, DEC-36).

#![forbid(unsafe_code)]

use std::process::ExitCode;

use zevrigo_demo::{Command, USAGE, init_logging, parse_args, run_rpc};

const BIN_NAME: &str = "zevrigo-demo";

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
        Ok(Command::Rpc(config)) => {
            init_logging();
            match run_rpc(&config) {
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
