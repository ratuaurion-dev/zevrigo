//! MOD-11: Daemon node Homeostasis Coin (`homeostasis-node`).

#![forbid(unsafe_code)]

use std::process::ExitCode;

const BIN_NAME: &str = "homeostasis-node";
const MODULE_ID: &str = "MOD-11";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match (args.next().as_deref(), args.next()) {
        (None, None) | (Some("--help"), None) => {
            println!("{BIN_NAME} ({MODULE_ID}): kerangka R0, belum berfungsi");
            ExitCode::SUCCESS
        }
        (Some("--version"), None) => {
            println!("{BIN_NAME} {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        (Some(arg), None) => {
            eprintln!("{BIN_NAME}: argumen tidak dikenali: {arg}");
            ExitCode::from(2)
        }
        (Some(_), Some(_)) | (None, Some(_)) => {
            eprintln!("{BIN_NAME}: argumen tidak valid (gunakan --help atau --version)");
            ExitCode::from(2)
        }
    }
}
