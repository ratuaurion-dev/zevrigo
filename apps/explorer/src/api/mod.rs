//! API HTTP baca saja explorer (Spesifikasi Explorer v0.1 Bagian 5), sebagai fungsi murni di atas [`Indexer`].
//!
//! API hanya menyajikan data sistem (DEC-28): blok, ringkasan transaksi, pasokan hasil tambang, dan jumlah mempool.
//! Alamat, saldo, jumlah per penerima, jejak koin (input, output, pembelanja), dan alokasi genesis tidak pernah
//! disajikan, supaya explorer tidak menampilkan "apa dan siapa". Datanya tetap ada di indeks untuk pemeriksaan I1–I4.
//!
//! [`respond`] menerima metode, jalur, dan query string, lalu mengembalikan kode status HTTP dan badan JSON. Server
//! axum ([`crate::server`]) hanya meneruskan permintaan ke fungsi ini, sehingga seluruh API bisa diuji tanpa jaringan.
//! Encoding nilai mengikuti Spesifikasi RPC v0.1 Bagian 2: jumlah koin string desimal quanta (tidak pernah angka
//! JSON), hash dan payload hex huruf kecil 64 karakter, target compact hex 8 karakter, nonce string desimal.

use serde_json::{Map, Value};
use zevrigo_primitives::{Amount, Hash32, Network};

use crate::index::{IndexError, Indexer};
use crate::query::confirmations;

/// Awalan semua jalur API.
pub const API_PREFIX: &str = "/api/v1/";

/// Jumlah blok paling banyak di `/blocks`.
pub const BLOCKS_PER_PAGE: usize = 20;

/// Badan jawaban galat internal.
const INTERNAL_BODY: &str = "{\"error\":\"internal\"}";

/// Ujung chain node menurut langkah sinkronisasi terakhir.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeView {
    /// Tinggi ujung node.
    pub height: u64,
    /// block_id ujung node.
    pub best_block_id: Hash32,
    /// Chainwork ujung node, 256 bit big-endian.
    pub chainwork: [u8; 32],
}

/// Keadaan sinkronisasi untuk `/status`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SyncStatus {
    /// Ujung node terakhir yang berhasil dibaca.
    pub node: Option<NodeView>,
    /// Galat langkah sinkronisasi terakhir (misalnya node tidak bisa dihubungi), `None` jika langkah itu berhasil.
    pub last_error: Option<String>,
}

/// Jawaban API: kode status HTTP dan badan JSON.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiResponse {
    /// Kode status HTTP.
    pub status: u16,
    /// Badan JSON (UTF-8).
    pub body: String,
}

impl ApiResponse {
    /// `500 {"error": "internal"}`.
    #[must_use]
    pub fn internal() -> Self {
        Self {
            status: 500,
            body: INTERNAL_BODY.to_owned(),
        }
    }
}

/// Galat permintaan.
enum ApiError {
    /// `400`: kunci, hex, atau angka query tidak sah.
    BadRequest,
    /// `404`: tidak ada di indeks; untuk transaksi, `in_mempool` ikut dilaporkan.
    NotFound { in_mempool: Option<bool> },
    /// `503`: genesis belum terindeks.
    NotReady,
    /// `405`: metode selain `GET`.
    MethodNotAllowed,
    /// `500`: indeks tidak bisa dibaca.
    Internal,
}

impl From<IndexError> for ApiError {
    fn from(_error: IndexError) -> Self {
        Self::Internal
    }
}

const fn not_found() -> ApiError {
    ApiError::NotFound { in_mempool: None }
}

fn object(fields: Vec<(&str, Value)>) -> Value {
    Value::Object(
        fields
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect::<Map<String, Value>>(),
    )
}

fn hash(id: &Hash32) -> Value {
    Value::String(id.to_string())
}

fn bytes32(bytes: &[u8; 32]) -> Value {
    hash(&Hash32::from_bytes(*bytes))
}

fn amount(value: Amount) -> Value {
    Value::String(value.as_quanta().to_string())
}

fn number(value: u64) -> Value {
    Value::from(value)
}

fn count(value: usize) -> Value {
    number(u64::try_from(value).unwrap_or(u64::MAX))
}

fn optional(value: Option<Value>) -> Value {
    value.unwrap_or(Value::Null)
}

fn compact(target: u32) -> Value {
    Value::String(format!("{target:08x}"))
}

const fn network_name(network: Network) -> &'static str {
    match network {
        Network::Mainnet => "mainnet",
        Network::Testnet => "testnet",
        Network::Regtest => "regtest",
    }
}

/// Bilangan desimal kanonis (tanpa tanda, tanpa nol di depan kecuali `0`).
fn parse_number(text: &str) -> Option<u64> {
    let canonical = !text.is_empty()
        && text.bytes().all(|byte| byte.is_ascii_digit())
        && (text == "0" || !text.starts_with('0'));
    if canonical { text.parse().ok() } else { None }
}

/// Hash hex huruf kecil tepat 64 karakter.
fn parse_hash(text: &str) -> Option<Hash32> {
    let lower_hex = text.len() == 64
        && text
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'));
    if lower_hex {
        Hash32::from_hex(text).ok()
    } else {
        None
    }
}

/// Nilai angka `key` di query string; `None` jika tidak ada.
fn query_number(query: Option<&str>, key: &str) -> Result<Option<u64>, ApiError> {
    let Some(query) = query else {
        return Ok(None);
    };
    for pair in query.split('&') {
        let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
        if name == key {
            return parse_number(value).map(Some).ok_or(ApiError::BadRequest);
        }
    }
    Ok(None)
}

fn encode(status: u16, value: &Value) -> ApiResponse {
    match serde_json::to_string(value) {
        Ok(body) => ApiResponse { status, body },
        Err(_) => ApiResponse::internal(),
    }
}

fn error_response(error: &ApiError) -> ApiResponse {
    let (status, name) = match error {
        ApiError::BadRequest => (400, "bad_request"),
        ApiError::NotFound { .. } => (404, "not_found"),
        ApiError::NotReady => (503, "not_ready"),
        ApiError::MethodNotAllowed => (405, "method_not_allowed"),
        ApiError::Internal => (500, "internal"),
    };
    let mut fields = vec![("error", Value::String(name.to_owned()))];
    if let ApiError::NotFound {
        in_mempool: Some(flag),
    } = error
    {
        fields.push(("in_mempool", Value::Bool(*flag)));
    }
    encode(status, &object(fields))
}

/// Menjawab satu permintaan API (Spesifikasi Explorer v0.1 Bagian 5).
///
/// `method` adalah metode HTTP (`GET`), `path` jalur tanpa query, `query` query string tanpa `?`.
#[must_use]
pub fn respond(
    index: &Indexer,
    sync: &SyncStatus,
    network: Network,
    method: &str,
    path: &str,
    query: Option<&str>,
) -> ApiResponse {
    match route(index, sync, network, method, path, query) {
        Ok(value) => encode(200, &value),
        Err(error) => error_response(&error),
    }
}

fn route(
    index: &Indexer,
    sync: &SyncStatus,
    network: Network,
    method: &str,
    path: &str,
    query: Option<&str>,
) -> Result<Value, ApiError> {
    if method != "GET" {
        return Err(ApiError::MethodNotAllowed);
    }
    let Some(rest) = path.strip_prefix(API_PREFIX) else {
        return Err(not_found());
    };
    let segments: Vec<&str> = rest.split('/').collect();
    if let ["status"] = segments.as_slice() {
        return status(index, sync, network);
    }
    let Some((tip, _)) = index.tip()? else {
        return Err(ApiError::NotReady);
    };
    let ctx = Ctx { index, tip };
    match segments.as_slice() {
        ["blocks"] => ctx.blocks(query_number(query, "before")?),
        ["block", key] => ctx.block(key),
        ["tx", id] => ctx.tx(id),
        ["supply"] => ctx.supply(),
        ["mempool"] => ctx.mempool(),
        ["search", text] => ctx.search(text),
        _ => Err(not_found()),
    }
}

fn status(index: &Indexer, sync: &SyncStatus, network: Network) -> Result<Value, ApiError> {
    let tip = index.tip()?;
    let node = sync.node;
    let mempool = index.mempool()?;
    Ok(object(vec![
        ("network", Value::String(network_name(network).to_owned())),
        (
            "indexed_height",
            optional(tip.map(|(height, _)| number(height))),
        ),
        ("indexed_block_id", optional(tip.map(|(_, id)| hash(&id)))),
        (
            "node_height",
            optional(node.map(|view| number(view.height))),
        ),
        (
            "node_best_block_id",
            optional(node.map(|view| hash(&view.best_block_id))),
        ),
        (
            "chainwork",
            optional(node.map(|view| bytes32(&view.chainwork))),
        ),
        ("mempool_count", count(mempool.len())),
        (
            "index_error",
            optional(index.halted().map(|error| Value::String(error.to_string()))),
        ),
        (
            "last_error",
            optional(sync.last_error.clone().map(Value::String)),
        ),
    ]))
}

/// Konteks satu permintaan sesudah genesis terindeks.
struct Ctx<'a> {
    index: &'a Indexer,
    tip: u64,
}

impl Ctx<'_> {
    fn confirmations(&self, height: u64) -> Value {
        number(confirmations(self.tip, height))
    }

    fn blocks(&self, before: Option<u64>) -> Result<Value, ApiError> {
        let rows = self.index.blocks_before(before, BLOCKS_PER_PAGE)?;
        let items = rows
            .iter()
            .map(|row| {
                object(vec![
                    ("height", number(row.height)),
                    ("block_id", hash(&row.block_id)),
                    ("timestamp", number(row.timestamp)),
                    ("size", number(row.size)),
                    ("tx_count", number(row.tx_count)),
                    ("confirmations", self.confirmations(row.height)),
                ])
            })
            .collect();
        Ok(object(vec![("blocks", Value::Array(items))]))
    }

    fn block(&self, key: &str) -> Result<Value, ApiError> {
        let detail = if let Some(height) = parse_number(key) {
            self.index.block_at(height)?
        } else if let Some(id) = parse_hash(key) {
            self.index.block_by_id(&id)?
        } else {
            return Err(ApiError::BadRequest);
        };
        let detail = detail.ok_or_else(not_found)?;
        let row = &detail.block;
        Ok(object(vec![
            ("block_id", hash(&row.block_id)),
            ("height", number(row.height)),
            ("prev_block_id", hash(&row.prev_block_id)),
            ("merkle_root", hash(&row.merkle_root)),
            ("timestamp", number(row.timestamp)),
            ("target", compact(row.target)),
            ("nonce", Value::String(row.nonce.to_string())),
            ("size", number(row.size)),
            ("tx_count", number(row.tx_count)),
            ("subsidy", amount(row.subsidy)),
            ("fees", amount(row.fees)),
            ("confirmations", self.confirmations(row.height)),
            (
                "txids",
                Value::Array(detail.txids.iter().map(hash).collect()),
            ),
            (
                "next_block_id",
                optional(detail.next_block_id.as_ref().map(hash)),
            ),
        ]))
    }

    fn tx(&self, text: &str) -> Result<Value, ApiError> {
        let id = parse_hash(text).ok_or(ApiError::BadRequest)?;
        let Some(detail) = self.index.tx(&id)? else {
            return Err(ApiError::NotFound {
                in_mempool: Some(self.index.in_mempool(&id)?),
            });
        };
        // Hanya ringkasan (DEC-28): tanpa alamat, jumlah per output, maupun rujukan input dan pembelanja.
        let tx = &detail.tx;
        Ok(object(vec![
            ("txid", hash(&tx.txid)),
            ("block_id", hash(&tx.block_id)),
            ("height", number(tx.height)),
            ("position", number(tx.position)),
            ("size", number(tx.size)),
            ("is_coinbase", Value::Bool(tx.is_coinbase)),
            ("fee", amount(tx.fee)),
            ("confirmations", self.confirmations(tx.height)),
            ("input_count", count(detail.inputs.len())),
            ("output_count", count(detail.outputs.len())),
        ]))
    }

    fn supply(&self) -> Result<Value, ApiError> {
        let info = self.index.supply()?.ok_or(ApiError::NotReady)?;
        Ok(object(vec![
            ("height", number(info.height)),
            ("mined_total", amount(info.mined_total)),
            ("scheduled_mined_total", amount(info.scheduled_mined_total)),
            ("matches_schedule", Value::Bool(info.matches_schedule)),
            ("fees_total", amount(info.fees_total)),
            ("next_subsidy", amount(info.next_subsidy)),
            ("tip_target", compact(info.tip_target)),
            (
                "average_block_seconds",
                optional(info.average_block_seconds.map(number)),
            ),
        ]))
    }

    fn mempool(&self) -> Result<Value, ApiError> {
        let txids = self.index.mempool()?;
        Ok(object(vec![("count", count(txids.len()))]))
    }

    fn search(&self, text: &str) -> Result<Value, ApiError> {
        let found = |kind: &str, key: Value| {
            object(vec![("kind", Value::String(kind.to_owned())), ("key", key)])
        };
        if let Some(height) = parse_number(text) {
            let detail = self.index.block_at(height)?.ok_or_else(not_found)?;
            return Ok(found("block", hash(&detail.block.block_id)));
        }
        if let Some(id) = parse_hash(text) {
            if self.index.block_by_id(&id)?.is_some() {
                return Ok(found("block", hash(&id)));
            }
            if self.index.tx(&id)?.is_some() {
                return Ok(found("tx", hash(&id)));
            }
            return Err(ApiError::NotFound {
                in_mempool: Some(self.index.in_mempool(&id)?),
            });
        }
        // Alamat tidak dicari (DEC-28): teks selain tinggi dan hash adalah permintaan tidak sah.
        Err(ApiError::BadRequest)
    }
}

#[cfg(test)]
mod tests;
