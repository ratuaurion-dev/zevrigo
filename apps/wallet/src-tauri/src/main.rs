//! MOD-15: wallet desktop Zevrigo Coin (`zevrigo-wallet`), TSK-M42 (DEC-31).
//!
//! Aplikasi Tauri 2: antarmuka React di `apps/wallet/src` memanggil perintah di [`commands`] lewat IPC Tauri. Semua
//! pekerjaan wallet (file wallet terenkripsi, alamat, saldo, riwayat, kirim, lihat frasa) memakai pustaka MOD-12
//! `zevrigo-cli`, sehingga file wallet sama dengan CLI (`<folder data>/<jaringan>/wallet.json`). Jumlah koin melewati
//! IPC sebagai teks angka quanta, tidak pernah sebagai bilangan pecahan. Hanya testnet dan regtest.

#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use std::process::ExitCode;

fn main() -> ExitCode {
    let Some(state) = commands::AppState::from_env() else {
        eprintln!(
            "zevrigo-wallet: folder data tidak ditemukan (atur {})",
            commands::DATADIR_ENV
        );
        return ExitCode::FAILURE;
    };
    let result = tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::wallet_info,
            commands::new_phrase,
            commands::restore_wallet,
            commands::receive_address,
            commands::new_address,
            commands::chain_status,
            commands::balance,
            commands::history,
            commands::send,
            commands::reveal_phrase,
        ])
        .run(tauri::generate_context!());
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("zevrigo-wallet: {error}");
            ExitCode::FAILURE
        }
    }
}
