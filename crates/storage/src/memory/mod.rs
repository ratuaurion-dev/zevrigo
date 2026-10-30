//! Implementasi acuan penyimpanan chain in-memory (`MemoryStore`).

use std::collections::{BTreeMap, BTreeSet};

use homeostasis_primitives::block::Block;
use homeostasis_primitives::hash::Hash32;
use homeostasis_primitives::height::BlockHeight;
use homeostasis_primitives::tx::TxBody;
use homeostasis_primitives::utxo::{OutPoint, UtxoEntry};

use crate::store::{BlockUndo, ChainStore, StoreError};

/// Penyimpanan chain aktif di dalam memori.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MemoryStore {
    utxos: BTreeMap<OutPoint, UtxoEntry>,
    blocks: BTreeMap<Hash32, Block>,
    active: Vec<Hash32>,
    undo: BTreeMap<Hash32, BlockUndo>,
}

impl ChainStore for MemoryStore {
    fn tip(&self) -> Result<Option<(BlockHeight, Hash32)>, StoreError> {
        let Some(&block_id) = self.active.last() else {
            return Ok(None);
        };
        let last_idx = self
            .active
            .len()
            .checked_sub(1)
            .ok_or(StoreError::EmptyChain)?;
        let h = u64::try_from(last_idx).map_err(|_| StoreError::HeightOverflow)?;
        Ok(Some((BlockHeight::new(h), block_id)))
    }

    fn block_id_at(&self, height: BlockHeight) -> Result<Option<Hash32>, StoreError> {
        let Ok(idx) = usize::try_from(height.get()) else {
            return Ok(None);
        };
        Ok(self.active.get(idx).copied())
    }

    fn block(&self, block_id: &Hash32) -> Result<Option<Block>, StoreError> {
        Ok(self.blocks.get(block_id).cloned())
    }

    fn utxo(&self, outpoint: &OutPoint) -> Result<Option<UtxoEntry>, StoreError> {
        Ok(self.utxos.get(outpoint).cloned())
    }

    fn utxos_by_payloads(
        &self,
        payloads: &[[u8; 32]],
    ) -> Result<Vec<(OutPoint, UtxoEntry)>, StoreError> {
        let wanted: BTreeSet<[u8; 32]> = payloads.iter().copied().collect();
        // BTreeMap sudah urut naik OutPoint, jadi hasil tidak perlu diurutkan ulang.
        Ok(self
            .utxos
            .iter()
            .filter(|(_, entry)| wanted.contains(entry.output().payload()))
            .map(|(op, entry)| (*op, entry.clone()))
            .collect())
    }

    fn undo_data(&self, block_id: &Hash32) -> Result<Option<BlockUndo>, StoreError> {
        Ok(self.undo.get(block_id).cloned())
    }

    fn apply_block(
        &mut self,
        block: &Block,
        block_id: Hash32,
        txids: &[Hash32],
    ) -> Result<(), StoreError> {
        // langkah 1: periksa kecocokan jumlah txid dengan jumlah transaksi
        let txs_len = block.txs().len();
        if txids.len() != txs_len {
            return Err(StoreError::TxidCountMismatch {
                txs: txs_len,
                txids: txids.len(),
            });
        }

        // langkah 2: periksa tinggi dan sambungan rantai
        let header_height = block.header().height().get();
        if self.active.is_empty() {
            if header_height != 0 {
                return Err(StoreError::UnexpectedHeight {
                    expected: 0,
                    actual: header_height,
                });
            }
        } else {
            let last_idx = self
                .active
                .len()
                .checked_sub(1)
                .ok_or(StoreError::EmptyChain)?;
            let last_height = u64::try_from(last_idx).map_err(|_| StoreError::HeightOverflow)?;
            let expected_height = last_height
                .checked_add(1)
                .ok_or(StoreError::HeightOverflow)?;
            if header_height != expected_height {
                return Err(StoreError::UnexpectedHeight {
                    expected: expected_height,
                    actual: header_height,
                });
            }
            let tip_block_id = self
                .active
                .get(last_idx)
                .copied()
                .ok_or(StoreError::EmptyChain)?;
            if block.header().prev_block_id() != tip_block_id {
                return Err(StoreError::PrevMismatch {
                    expected: tip_block_id,
                    actual: block.header().prev_block_id(),
                });
            }
        }

        // langkah 3: kumpulkan perubahan UTXO pada overlay lokal
        let mut created = BTreeMap::new();
        let mut created_order = Vec::new();
        let mut created_all = BTreeSet::new();
        let mut spent = Vec::new();
        let mut spent_set = BTreeSet::new();

        for (tx, txid) in block.txs().iter().zip(txids.iter()) {
            if let TxBody::Regular(body) = tx.body() {
                for input in body.inputs() {
                    let op = OutPoint::from_input(input);
                    if created.remove(&op).is_some() {
                        created_order.retain(|x| x != &op);
                    } else if !spent_set.contains(&op) && self.utxos.contains_key(&op) {
                        let entry = self
                            .utxos
                            .get(&op)
                            .cloned()
                            .ok_or(StoreError::MissingUtxo { outpoint: op })?;
                        spent.push((op, entry));
                        spent_set.insert(op);
                    } else {
                        return Err(StoreError::MissingUtxo { outpoint: op });
                    }
                }
            }

            for (j, output) in tx.body().outputs().iter().enumerate() {
                let output_index = u32::try_from(j).map_err(|_| StoreError::OutputIndexOverflow)?;
                let op = OutPoint::new(*txid, output_index);
                if self.utxos.contains_key(&op) || created_all.contains(&op) {
                    return Err(StoreError::DuplicateOutpoint { outpoint: op });
                }
                let entry =
                    UtxoEntry::new(output.clone(), block.header().height(), tx.is_coinbase());
                created.insert(op, entry);
                created_order.push(op);
                created_all.insert(op);
            }
        }

        // langkah 4: terapkan perubahan ke state permanen (tanpa langkah yang bisa gagal)
        for (op, _) in &spent {
            self.utxos.remove(op);
        }
        self.utxos.append(&mut created);
        self.blocks.insert(block_id, block.clone());
        self.active.push(block_id);
        self.undo
            .insert(block_id, BlockUndo::new(spent, created_order));

        Ok(())
    }

    fn undo_tip(&mut self) -> Result<(Hash32, Block), StoreError> {
        let tip_block_id = self.active.last().copied().ok_or(StoreError::EmptyChain)?;

        let undo = self
            .undo
            .get(&tip_block_id)
            .cloned()
            .ok_or(StoreError::CorruptUndo)?;
        let block = self
            .blocks
            .get(&tip_block_id)
            .cloned()
            .ok_or(StoreError::CorruptUndo)?;

        for op in undo.created() {
            if !self.utxos.contains_key(op) {
                return Err(StoreError::CorruptUndo);
            }
        }
        for (op, _) in undo.spent() {
            if self.utxos.contains_key(op) {
                return Err(StoreError::CorruptUndo);
            }
        }

        for op in undo.created() {
            self.utxos.remove(op);
        }
        for (op, entry) in undo.spent() {
            self.utxos.insert(*op, entry.clone());
        }
        self.active.pop();
        self.undo.remove(&tip_block_id);
        self.blocks.remove(&tip_block_id);

        Ok((tip_block_id, block))
    }
}

#[cfg(test)]
mod tests;
