//! Sumber data indexer dari node lewat JSON-RPC v0.1 (Spesifikasi Explorer v0.1 Bagian 1 dan 3): `get_chain_info`,
//! `get_block_hash`, `get_block`, dan `get_mempool`. Tanpa metode RPC baru.
//!
//! [`ChainSource`] sinkron, sedangkan klien RPC async: setiap panggilan dijalankan dengan `Handle::block_on` milik
//! runtime tokio. Karena itu [`RpcSource`] wajib dipanggil dari thread biasa atau dari `spawn_blocking`, tidak dari
//! dalam task async.

use std::fmt::Display;

use homeostasis_primitives::Hash32;
use homeostasis_rpc::{Hex32, RpcClient, error_code};
use tokio::runtime::Handle;

use crate::api::NodeView;
use crate::source::{ChainSource, SourceBlock, SourceError};

/// Kode error RPC "tidak ditemukan" (Spesifikasi RPC v0.1 Bagian 4).
const NOT_FOUND: i32 = 1001;

fn source_error(error: impl Display) -> SourceError {
    SourceError(error.to_string())
}

/// Node lewat RPC sebagai [`ChainSource`].
pub struct RpcSource {
    client: RpcClient,
    handle: Handle,
}

impl RpcSource {
    /// Sumber di atas `client`; panggilan RPC dijalankan di runtime `handle`.
    #[must_use]
    pub fn new(client: RpcClient, handle: Handle) -> Self {
        Self { client, handle }
    }

    /// Ujung chain node beserta chainwork-nya (`get_chain_info`), untuk `/status`.
    ///
    /// # Errors
    ///
    /// [`SourceError`] jika node tidak bisa dibaca.
    pub fn node_view(&self) -> Result<NodeView, SourceError> {
        let info = self
            .handle
            .block_on(self.client.get_chain_info())
            .map_err(source_error)?;
        Ok(NodeView {
            height: info.height,
            best_block_id: info.best_block_id.to_hash32(),
            chainwork: *info.chainwork.bytes(),
        })
    }
}

impl ChainSource for RpcSource {
    fn tip(&self) -> Result<(u64, Hash32), SourceError> {
        let view = self.node_view()?;
        Ok((view.height, view.best_block_id))
    }

    fn block_id_at(&self, height: u64) -> Result<Option<Hash32>, SourceError> {
        match self.handle.block_on(self.client.get_block_hash(height)) {
            Ok(id) => Ok(Some(id.to_hash32())),
            Err(error) if error_code(&error) == Some(NOT_FOUND) => Ok(None),
            Err(error) => Err(source_error(error)),
        }
    }

    fn block(&self, id: &Hash32) -> Result<Option<SourceBlock>, SourceError> {
        match self
            .handle
            .block_on(self.client.get_block(Hex32::from(*id)))
        {
            Ok(info) => Ok(Some(SourceBlock {
                bytes: info.hex.into_vec(),
                txids: info.txids.into_iter().map(Hex32::to_hash32).collect(),
            })),
            Err(error) if error_code(&error) == Some(NOT_FOUND) => Ok(None),
            Err(error) => Err(source_error(error)),
        }
    }

    fn mempool(&self) -> Result<Vec<Hash32>, SourceError> {
        let info = self
            .handle
            .block_on(self.client.get_mempool())
            .map_err(source_error)?;
        Ok(info.txids.into_iter().map(Hex32::to_hash32).collect())
    }
}

#[cfg(test)]
mod tests;
