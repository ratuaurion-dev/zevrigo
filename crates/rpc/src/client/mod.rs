//! Klien JSON-RPC lewat HTTP untuk CLI dan miner (Spesifikasi RPC v0.1), di atas `jsonrpsee-http-client` 0.26
//! tanpa TLS, karena server RPC hanya mendengarkan alamat lokal.

use jsonrpsee::core::ClientError;
use jsonrpsee::core::client::ClientT;
use jsonrpsee_http_client::{HttpClient, HttpClientBuilder};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use crate::error::RejectData;
use crate::transport::MAX_REQUEST_BYTES;
use crate::types::{
    BlockInfo, BlockTemplate, ChainInfo, GetBlockHashParams, GetBlockParams,
    GetBlockTemplateParams, METHOD_GET_BLOCK, METHOD_GET_BLOCK_HASH, METHOD_GET_BLOCK_TEMPLATE,
    METHOD_GET_CHAIN_INFO, METHOD_GET_MEMPOOL, METHOD_SCAN_UTXOS, METHOD_SEND_RAW_TRANSACTION,
    METHOD_SUBMIT_BLOCK, MempoolInfo, PayloadList, ScanUtxosParams, SendRawTransactionParams,
    SendRawTransactionResult, SubmitBlockParams, SubmitBlockResult, UtxoInfo,
};
use crate::value::{Hex32, HexBytes, KindNumber};

/// Klien untuk 8 metode RPC v0.1. Parameter selalu dikirim berdasarkan nama.
pub struct RpcClient {
    inner: HttpClient,
}

/// Mengubah parameter bertipe menjadi objek JSON.
fn object<T: Serialize>(params: &T) -> Result<Map<String, Value>, ClientError> {
    match serde_json::to_value(params)? {
        Value::Object(map) => Ok(map),
        _ => Err(ClientError::Custom(
            "parameter RPC harus berupa objek JSON".to_owned(),
        )),
    }
}

impl RpcClient {
    /// Membuat klien ke `url`, misalnya `http://127.0.0.1:28801`. Ukuran request dibatasi [`MAX_REQUEST_BYTES`].
    ///
    /// # Errors
    ///
    /// [`ClientError::Transport`] jika `url` tidak sah.
    pub fn new(url: &str) -> Result<Self, ClientError> {
        let inner = HttpClientBuilder::default()
            .max_request_size(MAX_REQUEST_BYTES)
            .build(url)?;
        Ok(Self { inner })
    }

    /// Memanggil `method` dengan parameter objek JSON.
    async fn call<R: DeserializeOwned>(
        &self,
        method: &str,
        params: Map<String, Value>,
    ) -> Result<R, ClientError> {
        self.inner.request(method, params).await
    }

    /// `get_chain_info`.
    ///
    /// # Errors
    ///
    /// [`ClientError::Call`] berisi kode error server, atau error transport.
    pub async fn get_chain_info(&self) -> Result<ChainInfo, ClientError> {
        self.call(METHOD_GET_CHAIN_INFO, Map::new()).await
    }

    /// `get_block_hash`.
    ///
    /// # Errors
    ///
    /// [`ClientError::Call`] berisi kode error server (`1001` jika tidak ada), atau error transport.
    pub async fn get_block_hash(&self, height: u64) -> Result<Hex32, ClientError> {
        let params = object(&GetBlockHashParams { height })?;
        self.call(METHOD_GET_BLOCK_HASH, params).await
    }

    /// `get_block`.
    ///
    /// # Errors
    ///
    /// [`ClientError::Call`] berisi kode error server (`1001` jika tidak ada), atau error transport.
    pub async fn get_block(&self, block_id: Hex32) -> Result<BlockInfo, ClientError> {
        let params = object(&GetBlockParams { block_id })?;
        self.call(METHOD_GET_BLOCK, params).await
    }

    /// `get_block_template`.
    ///
    /// # Errors
    ///
    /// [`ClientError::Call`] berisi kode error server, atau error transport.
    pub async fn get_block_template(
        &self,
        payout_kind: KindNumber,
        payout_payload: Hex32,
    ) -> Result<BlockTemplate, ClientError> {
        let params = object(&GetBlockTemplateParams {
            payout_kind,
            payout_payload,
        })?;
        self.call(METHOD_GET_BLOCK_TEMPLATE, params).await
    }

    /// `submit_block` dengan encoding kanonis blok.
    ///
    /// # Errors
    ///
    /// [`ClientError::Call`] berisi kode error server, atau error transport.
    pub async fn submit_block(&self, block: &[u8]) -> Result<SubmitBlockResult, ClientError> {
        let params = object(&SubmitBlockParams {
            hex: HexBytes::new(block.to_vec()),
        })?;
        self.call(METHOD_SUBMIT_BLOCK, params).await
    }

    /// `send_raw_transaction` dengan encoding kanonis transaksi lengkap.
    ///
    /// # Errors
    ///
    /// [`ClientError::Call`] berisi kode `1002` (lihat [`reject_reason`]) atau `1003`, atau error transport.
    pub async fn send_raw_transaction(
        &self,
        tx: &[u8],
    ) -> Result<SendRawTransactionResult, ClientError> {
        let params = object(&SendRawTransactionParams {
            hex: HexBytes::new(tx.to_vec()),
        })?;
        self.call(METHOD_SEND_RAW_TRANSACTION, params).await
    }

    /// `get_mempool`.
    ///
    /// # Errors
    ///
    /// [`ClientError::Call`] berisi kode error server, atau error transport.
    pub async fn get_mempool(&self) -> Result<MempoolInfo, ClientError> {
        self.call(METHOD_GET_MEMPOOL, Map::new()).await
    }

    /// `scan_utxos` untuk 1 sampai 64 payload.
    ///
    /// # Errors
    ///
    /// [`ClientError::Custom`] jika jumlah payload di luar 1 sampai 64 (tidak dikirim), [`ClientError::Call`]
    /// berisi kode error server, atau error transport.
    pub async fn scan_utxos(&self, payloads: &[Hex32]) -> Result<Vec<UtxoInfo>, ClientError> {
        let payloads = PayloadList::try_from(payloads.to_vec())
            .map_err(|e| ClientError::Custom(e.to_string()))?;
        let params = object(&ScanUtxosParams { payloads })?;
        self.call(METHOD_SCAN_UTXOS, params).await
    }

    /// Memanggil `method` dengan parameter JSON apa adanya (objek atau array), untuk alat diagnosis dan test.
    ///
    /// # Errors
    ///
    /// [`ClientError::Custom`] jika `params` bukan objek atau array, [`ClientError::Call`] berisi kode error
    /// server, atau error transport.
    pub async fn raw_request(&self, method: &str, params: Value) -> Result<Value, ClientError> {
        match params {
            Value::Object(map) => self.inner.request(method, map).await,
            Value::Array(items) => self.inner.request(method, items).await,
            _ => Err(ClientError::Custom(
                "parameter harus objek atau array JSON".to_owned(),
            )),
        }
    }
}

/// Kode error JSON-RPC jika server membalas dengan error, selain itu `None`.
#[must_use]
pub fn error_code(err: &ClientError) -> Option<i32> {
    match err {
        ClientError::Call(e) => Some(e.code()),
        _ => None,
    }
}

/// Nama aturan dari `data.reason` pada error `1002`, jika ada.
#[must_use]
pub fn reject_reason(err: &ClientError) -> Option<String> {
    let ClientError::Call(e) = err else {
        return None;
    };
    let data = e.data()?;
    serde_json::from_str::<RejectData>(data.get())
        .ok()
        .map(|d| d.reason)
}
