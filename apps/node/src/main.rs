//! MOD-11: Daemon node Zevrigo Coin (`zevrigo-node`).

#![forbid(unsafe_code)]

use std::process::ExitCode;

use zevrigo_node::daemon::{self, Command, USAGE};

const BIN_NAME: &str = "zevrigo-node";

fn main() -> ExitCode {
    match daemon::parse_args(std::env::args().skip(1)) {
        Ok(Command::Help) => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Ok(Command::Version) => {
            println!("{BIN_NAME} {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Ok(Command::Run(config)) => {
            daemon::init_logging();
            match daemon::run(&config) {
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
