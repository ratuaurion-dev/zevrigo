//! Sinkronisasi header lalu blok di sisi chain (TSK-M28, Spesifikasi P2P v0.1 Bagian 5): locator, jawaban
//! `GetHeaders` dan `GetBlocks`, indeks header tanpa badan blok, dan daftar badan blok yang perlu diminta.
//!
//! Pesan dan rekan diurus [`crate::p2p`]; modul ini hanya membaca dan mengubah state chain. Header diperiksa V2, V3,
//! dan V5 sampai V8 dengan fungsi MOD-04; V4 (Merkle root) dan V9 sampai V10 diperiksa saat badan blok tiba lewat
//! [`Node::receive_block`].

use std::collections::BTreeSet;

use homeostasis_consensus::{
    ParentContext, U256, check_future_timestamp, check_header_context, check_header_context_free,
};
use homeostasis_crypto::block_id;
use homeostasis_primitives::params::MEDIAN_TIME_SPAN;
use homeostasis_primitives::{Block, BlockHeight, Hash32, Header};
use homeostasis_rpc::RpcError;
use homeostasis_storage::ChainStore;

use super::{
    ChainState, HeaderEntry, INVALID_ANCESTOR_RULE, Node, Parent, block_rule, header_work, internal,
};

/// Jumlah ID locator paling banyak (Spesifikasi P2P Bagian 3).
pub const MAX_LOCATOR_IDS: usize = 64;

/// Jumlah ID pertama locator yang berjarak satu blok.
const DENSE_LOCATOR_IDS: usize = 10;

/// Tinggi blok locator untuk ujung setinggi `tip` (Spesifikasi P2P Bagian 5): `tip` sampai `tip − 9`, lalu jaraknya
/// digandakan (`tip − 11`, `tip − 15`, `tip − 23`, dan seterusnya), ditutup genesis. Paling banyak 64 tinggi, urut
/// turun.
#[must_use]
pub fn locator_heights(tip: u64) -> Vec<u64> {
    let mut heights = Vec::new();
    let mut height = tip;
    let mut step = 1_u64;
    loop {
        heights.push(height);
        if heights.len() >= DENSE_LOCATOR_IDS {
            step = step.saturating_mul(2);
        }
        match height.checked_sub(step) {
            Some(next) if next > 0 => height = next,
            _ => break,
        }
    }
    heights.truncate(MAX_LOCATOR_IDS.saturating_sub(1));
    if heights.last() != Some(&0) {
        heights.push(0);
    }
    heights
}

/// Hasil memproses satu pesan `Headers` (Spesifikasi P2P Bagian 5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HeadersOutcome {
    /// Semua header lolos atau sudah dikenal.
    Accepted {
        /// Jumlah header baru di indeks header.
        added: usize,
        /// ID header terakhir di pesan, `None` jika pesannya kosong.
        last: Option<Hash32>,
    },
    /// Header pertama tidak menyambung ke blok atau header yang dikenal, atau sebuah header tidak menyambung ke header
    /// sebelumnya. Rekan disanksi.
    Disconnected,
    /// Header tidak sah (V2, V3, V5 sampai V7, atau leluhurnya tidak sah). Rekan disanksi.
    Invalid {
        /// Nama aturan yang dilanggar.
        rule: &'static str,
    },
    /// Header gagal V8 (terlalu jauh di masa depan untuk jam lokal). Pemrosesan berhenti tanpa sanksi; header
    /// sebelumnya tetap disimpan.
    Deferred {
        /// Jumlah header baru sebelum header yang ditunda.
        added: usize,
    },
    /// Indeks header penuh (`NodeLimits::max_headers`) dan tidak ada header yang lebih ringan untuk dibuang.
    /// Pemrosesan berhenti tanpa sanksi; header sebelumnya tetap disimpan, dan sinkronisasi header berlanjut setelah
    /// badan blok diunduh.
    Full {
        /// Jumlah header baru sebelum indeks penuh.
        added: usize,
    },
}

impl<S> ChainState<S> {
    /// Menyimpan header di indeks header dengan nomor urut kedatangan berikutnya. Jika indeks penuh
    /// (`NodeLimits::max_headers`), header daun dengan chainwork paling kecil dibuang lebih dulu, asalkan chainwork-nya
    /// lebih kecil daripada header baru dan bukan induk header baru (jika sama, yang paling akhir diterima). Jika tidak
    /// ada, header baru tidak disimpan dan hasilnya `false`.
    fn store_header(&mut self, id: Hash32, header: Header, chainwork: U256) -> bool {
        if self.headers.len() >= self.limits.max_headers {
            let prev = header.prev_block_id();
            let parents: BTreeSet<Hash32> = self
                .headers
                .values()
                .map(|entry| entry.header.prev_block_id())
                .collect();
            let victim = self
                .headers
                .iter()
                .filter(|(other, entry)| {
                    **other != prev && !parents.contains(*other) && entry.chainwork < chainwork
                })
                .min_by(|(_, a), (_, b)| {
                    a.chainwork
                        .cmp(&b.chainwork)
                        .then(b.sequence.cmp(&a.sequence))
                })
                .map(|(other, _)| *other);
            let Some(victim) = victim else {
                return false;
            };
            self.headers.remove(&victim);
        }
        let sequence = self.next_sequence;
        self.next_sequence = sequence.saturating_add(1);
        self.headers.insert(
            id,
            HeaderEntry {
                header,
                chainwork,
                sequence,
            },
        );
        true
    }

    /// Data induk untuk header baru: seperti [`ChainState::parent_info`], tetapi induk dan leluhurnya juga boleh
    /// berupa header di indeks header.
    fn header_parent(&self, parent_id: &Hash32) -> Result<Option<Parent>, RpcError> {
        let Some(first) = self.headers.get(parent_id) else {
            return self.parent_info(parent_id);
        };
        let height = first.header.height().get();
        let chainwork = first.chainwork;
        let mut newest_first: Vec<u64> = Vec::with_capacity(MEDIAN_TIME_SPAN);
        let mut cursor = *parent_id;
        while newest_first.len() < MEDIAN_TIME_SPAN {
            if let Some(entry) = self.headers.get(&cursor) {
                newest_first.push(entry.header.timestamp());
                cursor = entry.header.prev_block_id();
                continue;
            }
            let need = MEDIAN_TIME_SPAN.saturating_sub(newest_first.len());
            let base = self
                .parent_info(&cursor)?
                .ok_or_else(|| internal("leluhur header tidak dikenal"))?;
            newest_first.extend(base.timestamps.iter().rev().take(need));
            break;
        }
        newest_first.reverse();
        Ok(Some(Parent {
            height,
            chainwork,
            timestamps: newest_first,
        }))
    }

    /// Header dengan chainwork terbanyak; jika sama, yang lebih dulu diterima.
    pub(super) fn best_header(&self) -> Option<(Hash32, U256)> {
        let mut best: Option<(Hash32, U256, u64)> = None;
        for (id, entry) in &self.headers {
            let better = best.is_none_or(|(_, work, sequence)| {
                entry.chainwork > work || (entry.chainwork == work && entry.sequence < sequence)
            });
            if better {
                best = Some((*id, entry.chainwork, entry.sequence));
            }
        }
        best.map(|(id, work, _)| (id, work))
    }

    /// Membuang header yang bloknya sudah dikenal atau tidak sah, lalu header yang induknya tidak sah (ditandai
    /// [`INVALID_ANCESTOR_RULE`]).
    fn prune_headers(&mut self) {
        let stale: Vec<Hash32> = self
            .headers
            .keys()
            .filter(|id| self.invalid.contains_key(*id) || self.known(id))
            .copied()
            .collect();
        for id in &stale {
            self.headers.remove(id);
        }
        let orphaned: Vec<Hash32> = self
            .headers
            .iter()
            .filter(|(_, entry)| self.invalid.contains_key(&entry.header.prev_block_id()))
            .map(|(id, _)| *id)
            .collect();
        if orphaned.is_empty() {
            return;
        }
        let mut dropped = self.descendants(&orphaned);
        dropped.extend(orphaned);
        for id in dropped {
            self.headers.remove(&id);
            self.note_invalid(id, INVALID_ANCESTOR_RULE);
        }
    }
}

impl<S: ChainStore> Node<S> {
    /// Locator chain aktif untuk `GetHeaders`, dari ujung ke genesis (lihat [`locator_heights`]).
    ///
    /// # Errors
    ///
    /// [`RpcError::Internal`] jika state node rusak.
    pub fn locator(&self) -> Result<Vec<Hash32>, RpcError> {
        let state = self.lock()?;
        let (tip, _) = state.tip()?;
        locator_heights(tip)
            .into_iter()
            .map(|height| {
                let index = usize::try_from(height).map_err(internal)?;
                state
                    .metas
                    .get(index)
                    .map(|meta| meta.id)
                    .ok_or_else(|| internal("ringkasan chain tidak lengkap"))
            })
            .collect()
    }

    /// Jawaban `GetHeaders`: header chain aktif sesudah ID locator pertama yang ada di chain aktif, berurutan, paling
    /// banyak `max`, berhenti di ujung atau sesudah blok `stop` (nol berarti tanpa batas). Kosong jika tidak ada ID
    /// locator yang dikenal.
    ///
    /// # Errors
    ///
    /// [`RpcError::Internal`] jika penyimpanan gagal dibaca.
    pub fn headers_after(
        &self,
        locator: &[Hash32],
        stop: &Hash32,
        max: usize,
    ) -> Result<Vec<Header>, RpcError> {
        let state = self.lock()?;
        let Some(fork) = locator.iter().find_map(|id| state.heights.get(id).copied()) else {
            return Ok(Vec::new());
        };
        let start = usize::try_from(fork).map_err(internal)?.saturating_add(1);
        let mut headers = Vec::new();
        for meta in state.metas.iter().skip(start).take(max) {
            let block = state
                .store
                .block(&meta.id)
                .map_err(internal)?
                .ok_or_else(|| internal("blok chain aktif tidak ada di penyimpanan"))?;
            headers.push(*block.header());
            if meta.id == *stop {
                break;
            }
        }
        Ok(headers)
    }

    /// Jawaban `GetBlocks`: blok untuk setiap ID (chain aktif atau cabang; badan blok cabang dibaca dari penyimpanan),
    /// `None` jika node tidak punya.
    ///
    /// # Errors
    ///
    /// [`RpcError::Internal`] jika penyimpanan gagal dibaca.
    pub fn blocks_by_id(&self, ids: &[Hash32]) -> Result<Vec<Option<Block>>, RpcError> {
        let state = self.lock()?;
        let mut blocks = Vec::with_capacity(ids.len());
        for id in ids {
            let block = if state.side.contains_key(id) {
                state.store.side_block(id).map_err(internal)?
            } else if state.heights.contains_key(id) {
                state.store.block(id).map_err(internal)?
            } else {
                None
            };
            blocks.push(block);
        }
        Ok(blocks)
    }

    /// Memproses isi pesan `Headers` (Spesifikasi P2P Bagian 5): setiap header wajib menyambung ke header sebelumnya,
    /// header baru wajib menyambung ke blok atau header yang dikenal, lalu diperiksa V2, V3, dan V5 sampai V8 terhadap
    /// induknya. Header yang lolos masuk indeks header beserta chainwork kumulatifnya.
    ///
    /// # Errors
    ///
    /// [`RpcError::Internal`] jika state node rusak.
    pub fn accept_headers(&self, headers: &[Header]) -> Result<HeadersOutcome, RpcError> {
        let now = (self.clock)();
        let mut guard = self.lock()?;
        let state = &mut *guard;
        let mut added = 0_usize;
        let mut previous: Option<Hash32> = None;
        for header in headers {
            let id = block_id(header);
            let prev = header.prev_block_id();
            if let Some(expected) = previous
                && prev != expected
            {
                return Ok(HeadersOutcome::Disconnected);
            }
            previous = Some(id);
            if let Some(&rule) = state.invalid.get(&id) {
                return Ok(HeadersOutcome::Invalid { rule });
            }
            if state.invalid.contains_key(&prev) {
                return Ok(HeadersOutcome::Invalid {
                    rule: INVALID_ANCESTOR_RULE,
                });
            }
            if state.known(&id) || state.headers.contains_key(&id) {
                continue;
            }
            let Some(parent) = state.header_parent(&prev)? else {
                return Ok(HeadersOutcome::Disconnected);
            };
            if let Err(error) = check_header_context_free(header, &self.params) {
                return Ok(HeadersOutcome::Invalid {
                    rule: block_rule(&error, "V2"),
                });
            }
            let context = ParentContext {
                block_id: prev,
                height: BlockHeight::new(parent.height),
                timestamps: &parent.timestamps,
            };
            if let Err(error) = check_header_context(header, &context, &self.params) {
                return Ok(HeadersOutcome::Invalid {
                    rule: block_rule(&error, "V6"),
                });
            }
            if check_future_timestamp(header, now).is_err() {
                return Ok(HeadersOutcome::Deferred { added });
            }
            let work = header_work(header).ok_or_else(|| internal("target header tidak sah"))?;
            let chainwork = parent
                .chainwork
                .checked_add(work)
                .ok_or_else(|| internal("chainwork meluap"))?;
            if !state.store_header(id, *header, chainwork) {
                return Ok(HeadersOutcome::Full { added });
            }
            added = added.saturating_add(1);
        }
        Ok(HeadersOutcome::Accepted {
            added,
            last: previous,
        })
    }

    /// ID blok yang badannya perlu diminta (Spesifikasi P2P Bagian 5 langkah 4): jika header dengan chainwork terbanyak
    /// melebihi chainwork ujung chain aktif, jalur header-saja menuju header itu, urut dari yang terendah, paling banyak
    /// `max`. Header yang bloknya sudah dikenal atau tidak sah dibuang dulu dari indeks.
    ///
    /// # Errors
    ///
    /// [`RpcError::Internal`] jika state node rusak.
    pub fn missing_blocks(&self, max: usize) -> Result<Vec<Hash32>, RpcError> {
        let mut guard = self.lock()?;
        let state = &mut *guard;
        state.prune_headers();
        let (_, tip) = state.tip()?;
        let Some((best, chainwork)) = state.best_header() else {
            return Ok(Vec::new());
        };
        if chainwork <= tip.chainwork {
            return Ok(Vec::new());
        }
        let mut path = Vec::new();
        let mut cursor = best;
        while let Some(entry) = state.headers.get(&cursor) {
            path.push(cursor);
            cursor = entry.header.prev_block_id();
        }
        path.reverse();
        path.truncate(max);
        Ok(path)
    }
}
