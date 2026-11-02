//! Server JSON-RPC lewat HTTP (Spesifikasi RPC v0.1 Bagian 1, 3, dan 4) di atas `jsonrpsee` 0.26.
//!
//! MOD-08 tidak mengenal node. Node (MOD-11) menyediakan data lewat trait [`RpcBackend`]; server ini membaca
//! parameter (aturan encoding Bagian 2), memanggil backend, lalu menulis hasil atau kode error. Parameter wajib
//! berupa objek JSON berdasarkan nama; parameter yang tidak sah dijawab `-32602`.

use std::net::SocketAddr;
use std::sync::Arc;

use jsonrpsee::server::types::{ErrorObject, ErrorObjectOwned, Params};
use jsonrpsee::server::{RpcModule, Server, ServerConfig, ServerHandle};
use serde::de::DeserializeOwned;
use serde_json::Value;
use thiserror::Error;

use crate::error::{INVALID_PARAMS, RpcError};
use crate::transport::MAX_REQUEST_BYTES;
use crate::types::{
    BlockInfo, BlockTemplate, ChainInfo, GetBlockHashParams, GetBlockParams,
    GetBlockTemplateParams, METHOD_GET_BLOCK, METHOD_GET_BLOCK_HASH, METHOD_GET_BLOCK_TEMPLATE,
    METHOD_GET_CHAIN_INFO, METHOD_GET_MEMPOOL, METHOD_SCAN_MEMPOOL, METHOD_SCAN_UTXOS,
    METHOD_SEND_RAW_TRANSACTION, METHOD_SUBMIT_BLOCK, MempoolInfo, MempoolScan, ScanMempoolParams,
    ScanUtxosParams, SendRawTransactionParams, SendRawTransactionResult, SubmitBlockParams,
    SubmitBlockResult, UtxoInfo,
};
use crate::value::Hex32;

/// Sumber data node untuk 9 metode RPC. Setiap metode dipanggil dengan parameter yang sudah lolos aturan encoding.
pub trait RpcBackend: Send + Sync + 'static {
    /// `get_chain_info`: ringkasan ujung chain aktif.
    ///
    /// # Errors
    ///
    /// [`RpcError::Internal`] jika data node tidak bisa dibaca.
    fn chain_info(&self) -> Result<ChainInfo, RpcError>;

    /// `get_block_hash`: ID blok di chain aktif pada `height`.
    ///
    /// # Errors
    ///
    /// [`RpcError::NotFound`] jika tinggi itu belum ada.
    fn block_hash(&self, height: u64) -> Result<Hex32, RpcError>;

    /// `get_block`: isi blok di chain aktif.
    ///
    /// # Errors
    ///
    /// [`RpcError::NotFound`] jika blok tidak ada di chain aktif.
    fn block(&self, block_id: Hex32) -> Result<BlockInfo, RpcError>;

    /// `get_block_template`: blok siap ditambang untuk penerima hadiah di `params`.
    ///
    /// # Errors
    ///
    /// [`RpcError::Internal`] jika template tidak bisa disusun.
    fn block_template(&self, params: &GetBlockTemplateParams) -> Result<BlockTemplate, RpcError>;

    /// `submit_block`: memproses byte blok. Hasil `invalid` adalah hasil biasa, bukan error.
    ///
    /// # Errors
    ///
    /// [`RpcError::Internal`] jika node gagal memproses blok.
    fn submit_block(&self, block: &[u8]) -> Result<SubmitBlockResult, RpcError>;

    /// `send_raw_transaction`: memasukkan transaksi ke mempool.
    ///
    /// # Errors
    ///
    /// [`RpcError::InvalidEncoding`] jika byte bukan encoding kanonis, [`RpcError::InvalidTransaction`] jika
    /// transaksi melanggar aturan T1 sampai T6.
    fn send_raw_transaction(&self, tx: &[u8]) -> Result<SendRawTransactionResult, RpcError>;

    /// `get_mempool`: txid di mempool, urut naik bytewise.
    ///
    /// # Errors
    ///
    /// [`RpcError::Internal`] jika mempool tidak bisa dibaca.
    fn mempool(&self) -> Result<MempoolInfo, RpcError>;

    /// `scan_utxos`: UTXO di ujung chain aktif untuk 1 sampai 64 payload, urut `(txid, index)`.
    ///
    /// # Errors
    ///
    /// [`RpcError::Internal`] jika set UTXO tidak bisa dibaca.
    fn scan_utxos(&self, payloads: &[Hex32]) -> Result<Vec<UtxoInfo>, RpcError>;

    /// `scan_mempool`: untuk 1 sampai 64 payload, UTXO di ujung chain aktif yang dibelanjakan transaksi mempool dan
    /// output transaksi mempool ke payload itu, masing-masing urut `(txid, index)`.
    ///
    /// # Errors
    ///
    /// [`RpcError::Internal`] jika set UTXO atau mempool tidak bisa dibaca.
    fn scan_mempool(&self, payloads: &[Hex32]) -> Result<MempoolScan, RpcError>;
}

/// Error saat menyiapkan atau menjalankan server RPC.
#[derive(Debug, Error)]
pub enum ServerError {
    /// Metode gagal didaftarkan (misalnya nama ganda).
    #[error("gagal mendaftarkan metode RPC: {message}")]
    Register {
        /// Keterangan dari jsonrpsee.
        message: String,
    },
    /// Alamat server tidak bisa dibuka.
    #[error("gagal membuka alamat server RPC: {0}")]
    Io(#[from] std::io::Error),
}

/// Error `-32602` dengan keterangan di `data`.
fn invalid_params(detail: String) -> ErrorObjectOwned {
    ErrorObject::owned(INVALID_PARAMS, "Invalid params", Some(detail))
}

/// Membaca parameter berdasarkan nama. Selain objek JSON (array posisional, `null`, atau tanpa parameter) ditolak.
fn named<T: DeserializeOwned>(params: &Params<'_>) -> Result<T, ErrorObjectOwned> {
    let raw = match params.as_str() {
        Some(raw) if params.is_object() => raw,
        _ => {
            return Err(invalid_params(
                "parameter wajib berupa objek JSON berdasarkan nama".to_owned(),
            ));
        }
    };
    serde_json::from_str(raw).map_err(|e| invalid_params(e.to_string()))
}

/// Metode tanpa parameter hanya menerima parameter kosong: tidak ada, `null`, `{}`, atau `[]`.
fn no_params(params: &Params<'_>) -> Result<(), ErrorObjectOwned> {
    let Some(raw) = params.as_str() else {
        return Ok(());
    };
    match serde_json::from_str::<Value>(raw) {
        Ok(Value::Null) => Ok(()),
        Ok(Value::Object(map)) if map.is_empty() => Ok(()),
        Ok(Value::Array(items)) if items.is_empty() => Ok(()),
        Ok(_) => Err(invalid_params(
            "metode ini tidak menerima parameter".to_owned(),
        )),
        Err(e) => Err(invalid_params(e.to_string())),
    }
}

/// Menyusun modul berisi 9 metode RPC v0.1 di atas `backend`.
///
/// # Errors
///
/// [`ServerError::Register`] jika pendaftaran metode gagal.
pub fn rpc_module<B: RpcBackend>(backend: Arc<B>) -> Result<RpcModule<B>, ServerError> {
    let mut module = RpcModule::from_arc(backend);
    let register = |e: jsonrpsee::core::RegisterMethodError| ServerError::Register {
        message: e.to_string(),
    };
    module
        .register_method(
            METHOD_GET_CHAIN_INFO,
            |params, backend, _| -> Result<ChainInfo, ErrorObjectOwned> {
                no_params(&params)?;
                backend.chain_info().map_err(ErrorObjectOwned::from)
            },
        )
        .map_err(register)?;
    module
        .register_method(
            METHOD_GET_BLOCK_HASH,
            |params, backend, _| -> Result<Hex32, ErrorObjectOwned> {
                let p: GetBlockHashParams = named(&params)?;
                backend.block_hash(p.height).map_err(ErrorObjectOwned::from)
            },
        )
        .map_err(register)?;
    module
        .register_method(
            METHOD_GET_BLOCK,
            |params, backend, _| -> Result<BlockInfo, ErrorObjectOwned> {
                let p: GetBlockParams = named(&params)?;
                backend.block(p.block_id).map_err(ErrorObjectOwned::from)
            },
        )
        .map_err(register)?;
    module
        .register_method(
            METHOD_GET_BLOCK_TEMPLATE,
            |params, backend, _| -> Result<BlockTemplate, ErrorObjectOwned> {
                let p: GetBlockTemplateParams = named(&params)?;
                backend.block_template(&p).map_err(ErrorObjectOwned::from)
            },
        )
        .map_err(register)?;
    module
        .register_method(
            METHOD_SUBMIT_BLOCK,
            |params, backend, _| -> Result<SubmitBlockResult, ErrorObjectOwned> {
                let p: SubmitBlockParams = named(&params)?;
                backend
                    .submit_block(p.hex.as_slice())
                    .map_err(ErrorObjectOwned::from)
            },
        )
        .map_err(register)?;
    module
        .register_method(
            METHOD_SEND_RAW_TRANSACTION,
            |params, backend, _| -> Result<SendRawTransactionResult, ErrorObjectOwned> {
                let p: SendRawTransactionParams = named(&params)?;
                backend
                    .send_raw_transaction(p.hex.as_slice())
                    .map_err(ErrorObjectOwned::from)
            },
        )
        .map_err(register)?;
    module
        .register_method(
            METHOD_GET_MEMPOOL,
            |params, backend, _| -> Result<MempoolInfo, ErrorObjectOwned> {
                no_params(&params)?;
                backend.mempool().map_err(ErrorObjectOwned::from)
            },
        )
        .map_err(register)?;
    module
        .register_method(
            METHOD_SCAN_UTXOS,
            |params, backend, _| -> Result<Vec<UtxoInfo>, ErrorObjectOwned> {
                let p: ScanUtxosParams = named(&params)?;
                backend
                    .scan_utxos(p.payloads.as_slice())
                    .map_err(ErrorObjectOwned::from)
            },
        )
        .map_err(register)?;
    module
        .register_method(
            METHOD_SCAN_MEMPOOL,
            |params, backend, _| -> Result<MempoolScan, ErrorObjectOwned> {
                let p: ScanMempoolParams = named(&params)?;
                backend
                    .scan_mempool(p.payloads.as_slice())
                    .map_err(ErrorObjectOwned::from)
            },
        )
        .map_err(register)?;
    Ok(module)
}

/// Server RPC yang sedang berjalan di runtime tokio pemanggil.
pub struct RpcServer {
    handle: ServerHandle,
    local_addr: SocketAddr,
}

impl RpcServer {
    /// Membuka `addr` dan menjalankan server HTTP JSON-RPC (tanpa WebSocket) dengan batas request
    /// [`MAX_REQUEST_BYTES`]. Port `0` memilih port bebas; lihat [`RpcServer::local_addr`].
    ///
    /// # Errors
    ///
    /// [`ServerError::Io`] jika alamat tidak bisa dibuka, [`ServerError::Register`] jika modul gagal disusun.
    pub async fn start<B: RpcBackend>(
        addr: SocketAddr,
        backend: Arc<B>,
    ) -> Result<Self, ServerError> {
        let module = rpc_module(backend)?;
        let config = ServerConfig::builder()
            .max_request_body_size(MAX_REQUEST_BYTES)
            .http_only()
            .build();
        let server = Server::builder().set_config(config).build(addr).await?;
        let local_addr = server.local_addr()?;
        let handle = server.start(module);
        Ok(Self { handle, local_addr })
    }

    /// Alamat yang benar-benar dipakai server.
    #[must_use]
    pub const fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Menghentikan server dan menunggu sampai berhenti.
    pub async fn stop(self) {
        // Hasil diabaikan: AlreadyStoppedError berarti server memang sudah berhenti.
        let _ = self.handle.stop();
        self.handle.stopped().await;
    }
}

#[cfg(test)]
mod tests;
