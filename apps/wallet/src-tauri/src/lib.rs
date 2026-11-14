//! MOD-15: wallet Zevrigo Coin (`zevrigo-wallet`) untuk desktop (TSK-M42, DEC-31) dan Android (TSK-M58).
//!
//! Aplikasi Tauri 2: antarmuka React di `apps/wallet/src` memanggil perintah IPC di modul `commands`. Semua pekerjaan
//! wallet (file wallet terenkripsi, alamat, saldo, riwayat, kirim, lihat frasa) memakai pustaka MOD-12 `zevrigo-cli`,
//! sehingga file wallet sama dengan CLI (`<folder data>/<jaringan>/wallet.json`). Jumlah koin melewati IPC sebagai teks
//! angka quanta, tidak pernah sebagai bilangan pecahan. Hanya testnet dan regtest.
//!
//! - Desktop: program `zevrigo-wallet` (`main.rs`) memanggil [`try_run`]. Folder data dari `ZEVRIGO_WALLET_DATADIR`,
//!   atau bawaan `%APPDATA%\Zevrigo` (Windows) dan `~/.zevrigo` (sistem lain).
//! - Android: titik masuk aplikasi adalah [`run`] (atribut `tauri::mobile_entry_point`). Folder data adalah folder data
//!   aplikasi itu sendiri (`app_data_dir`), yang hanya bisa dibaca aplikasi ini dan ikut terhapus saat aplikasi dihapus.
//!   Sertifikat HTTPS gerbang RPC diperiksa terhadap daftar akar sistem Android (rustls-platform-verifier, pilihan
//!   pencipta 14 November 2026), yang disiapkan sekali saat aplikasi mulai lewat JNI aman dari WebView.

#![forbid(unsafe_code)]

mod commands;

/// Menjalankan aplikasi wallet sampai jendelanya ditutup.
///
/// # Errors
///
/// [`tauri::Error`] jika folder data tidak ditemukan (`Setup`) atau jendela dan WebView gagal dibuat.
pub fn try_run() -> Result<(), tauri::Error> {
    tauri::Builder::default()
        .setup(setup)
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
        .run(tauri::generate_context!())
}

/// Titik masuk aplikasi Android (dan pemanggilan dari desktop tanpa kode keluar): seperti [`try_run`], galat dicatat ke
/// stderr (logcat di Android).
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(error) = try_run() {
        eprintln!("zevrigo-wallet: {error}");
    }
}

/// Menyiapkan folder data (dan TLS Android) sebelum perintah IPC pertama.
fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::Manager as _;

    let state = data_state(app)?;
    app.manage(state);
    #[cfg(target_os = "android")]
    init_android_tls(app);
    Ok(())
}

/// Folder data desktop: [`commands::AppState::from_env`].
#[cfg(not(target_os = "android"))]
fn data_state(_app: &tauri::App) -> Result<commands::AppState, Box<dyn std::error::Error>> {
    commands::AppState::from_env().ok_or_else(|| {
        format!(
            "folder data tidak ditemukan (atur {})",
            commands::DATADIR_ENV
        )
        .into()
    })
}

/// Folder data Android: folder data aplikasi (`app_data_dir`).
#[cfg(target_os = "android")]
fn data_state(app: &tauri::App) -> Result<commands::AppState, Box<dyn std::error::Error>> {
    use tauri::Manager as _;

    let dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&dir)?;
    Ok(commands::AppState::new(dir))
}

/// Menyiapkan rustls-platform-verifier dengan konteks Activity dari WebView utama, lewat JNI yang disediakan Tauri.
/// Berjalan di utas Android; bila gagal, galat dicatat dan koneksi HTTPS akan gagal (wallet menampilkan status offline).
#[cfg(target_os = "android")]
fn init_android_tls(app: &tauri::App) {
    use tauri::Manager as _;

    let Some(window) = app.get_webview_window("main") else {
        eprintln!("zevrigo-wallet: jendela utama tidak ada; TLS Android tidak disiapkan");
        return;
    };
    let result = window.with_webview(|webview| {
        webview.jni_handle().exec(|env, activity, _webview| {
            let context = match env.new_local_ref(activity) {
                Ok(context) => context,
                Err(error) => {
                    eprintln!("zevrigo-wallet: konteks Android tidak didapat: {error}");
                    return;
                }
            };
            if let Err(error) = rustls_platform_verifier::android::init_with_env(env, context) {
                eprintln!("zevrigo-wallet: TLS Android gagal disiapkan: {error}");
            }
        });
    });
    if let Err(error) = result {
        eprintln!("zevrigo-wallet: WebView utama tidak bisa dipakai untuk TLS Android: {error}");
    }
}
