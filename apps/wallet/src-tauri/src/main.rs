//! MOD-15: program wallet desktop Zevrigo Coin (`zevrigo-wallet`), TSK-M42 (DEC-31). Isi aplikasi ada di pustaka
//! `zevrigo_wallet_lib` (juga titik masuk Android, TSK-M58).

#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::ExitCode;

fn main() -> ExitCode {
    match zevrigo_wallet_lib::try_run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("zevrigo-wallet: {error}");
            ExitCode::FAILURE
        }
    }
}
