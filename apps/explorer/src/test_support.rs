//! Chain buatan untuk test MOD-14: blok dengan coinbase tepat jadwal pasokan, tanpa PoW dan tanpa tanda tangan sah
//! (explorer tidak memeriksa keduanya), sumber data palsu dengan gangguan yang bisa dipilih, dan node tiruan di
//! belakang server RPC sungguhan.

use std::sync::Mutex;

use homeostasis_consensus::{block_subsidy, regtest_genesis, total_supply_after};
use homeostasis_crypto::{block_id, block_merkle_root, txid};
use homeostasis_primitives::{
    Amount, Block, BlockHeight, Encode, Hash32, Header, Input, Network, Output, OutputKind,
    Transaction, Witness,
};
use homeostasis_rpc::{
    BlockInfo, BlockTemplate, ChainInfo, CompactHex, GetBlockTemplateParams, Hex32, MempoolInfo,
    MempoolScan, NetworkName, Quanta, RpcBackend, RpcError, SendRawTransactionResult,
    SubmitBlockResult, UtxoInfo,
};

use crate::source::{ChainSource, SourceBlock, SourceError};

/// Cap waktu genesis regtest.
pub(crate) const T0: u64 = 1_793_145_600;

/// Target compact `pow_limit` regtest.
pub(crate) const LIMIT: u32 = 0x207f_ffff;

/// ID palsu yang dipakai gangguan [`Fault::WrongId`] dan [`Fault::WrongTxids`].
pub(crate) const FAKE_ID: Hash32 = Hash32::from_bytes([0xee; 32]);

/// Payload pemilik `owner`.
pub(crate) const fn payload(owner: u8) -> [u8; 32] {
    [owner; 32]
}

/// Output kunci tunggal `amount` quanta ke `owner`.
pub(crate) fn out(amount: u128, owner: u8) -> Output {
    Output::new(
        Amount::from_quanta(amount),
        OutputKind::SingleKey,
        payload(owner),
    )
    .unwrap()
}

pub(crate) fn id_of(block: &Block) -> Hash32 {
    block_id(block.header())
}

/// Txid coinbase genesis dan jumlah output pertamanya (alokasi genesis).
pub(crate) fn genesis_output() -> (Hash32, u128, [u8; 32]) {
    let genesis = regtest_genesis().unwrap();
    let coinbase = &genesis.txs()[0];
    let output = &coinbase.body().outputs()[0];
    (
        txid(coinbase),
        output.amount().as_quanta(),
        *output.payload(),
    )
}

/// Transaksi biasa yang membelanjakan `inputs` ke `outputs`, dengan witness bohongan.
pub(crate) fn spend(inputs: &[(Hash32, u32)], outputs: Vec<Output>) -> Transaction {
    let inputs: Vec<Input> = inputs
        .iter()
        .map(|(prev, index)| Input::new(*prev, *index))
        .collect();
    let witnesses = inputs
        .iter()
        .map(|_| Witness::single_key([0; 64]))
        .collect();
    Transaction::regular(inputs, outputs, witnesses).unwrap()
}

/// Chain aktif buatan, mulai dari genesis regtest.
#[derive(Clone)]
pub(crate) struct Chain {
    pub(crate) blocks: Vec<Block>,
    mined: Vec<Amount>,
}

impl Chain {
    pub(crate) fn new() -> Self {
        Self {
            blocks: vec![regtest_genesis().unwrap()],
            mined: vec![Amount::ZERO],
        }
    }

    pub(crate) fn height(&self) -> u64 {
        u64::try_from(self.blocks.len()).unwrap() - 1
    }

    pub(crate) fn id(&self, height: u64) -> Hash32 {
        id_of(&self.blocks[usize::try_from(height).unwrap()])
    }

    pub(crate) fn mined_total(&self) -> Amount {
        *self.mined.last().unwrap()
    }

    /// Blok baru ke `owner`: coinbase = subsidy + `fee` + `bonus` (bonus bukan nol membuat blok tidak sah, I4).
    pub(crate) fn push_with(
        &mut self,
        owner: u8,
        txs: Vec<Transaction>,
        fee: u128,
        bonus: u128,
    ) -> Hash32 {
        let height = self.height() + 1;
        let prev = self.id(height - 1);
        let mined = self.mined_total();
        let subsidy = block_subsidy(BlockHeight::new(height), mined).unwrap();
        let reward = subsidy.as_quanta() + fee + bonus;
        let coinbase = Transaction::coinbase(
            BlockHeight::new(height),
            Vec::new(),
            vec![out(reward, owner)],
        )
        .unwrap();
        let mut all = vec![coinbase];
        all.extend(txs);
        let draft = Header::new(
            BlockHeight::new(height),
            prev,
            Hash32::ZERO,
            T0 + 120 * height,
            LIMIT,
            0,
        );
        // Merkle root yang benar, agar blok cabang di tinggi yang sama punya block_id berbeda.
        let merkle = block_merkle_root(&Block::new(draft, all.clone()).unwrap()).unwrap();
        let header = Header::new(
            BlockHeight::new(height),
            prev,
            merkle,
            T0 + 120 * height,
            LIMIT,
            0,
        );
        let block = Block::new(header, all).unwrap();
        let id = id_of(&block);
        self.blocks.push(block);
        self.mined.push(mined.checked_add(subsidy).unwrap());
        id
    }

    pub(crate) fn push(&mut self, owner: u8, txs: Vec<Transaction>, fee: u128) -> Hash32 {
        self.push_with(owner, txs, fee, 0)
    }

    /// Salinan chain sampai tinggi `height` (untuk cabang).
    pub(crate) fn fork_at(&self, height: u64) -> Self {
        let keep = usize::try_from(height).unwrap() + 1;
        Self {
            blocks: self.blocks[..keep].to_vec(),
            mined: self.mined[..keep].to_vec(),
        }
    }

    pub(crate) fn source(&self) -> FakeSource<'_> {
        FakeSource {
            chain: self,
            mempool: Vec::new(),
            fault: Fault::None,
        }
    }
}

/// Gangguan sumber data palsu.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Fault {
    None,
    /// Semua panggilan gagal.
    Offline,
    /// `block_id_at(h)` memberi [`FAKE_ID`], dan blok `h` dikirim untuk ID itu (I1).
    WrongId(u64),
    /// Txid pertama blok `h` diganti [`FAKE_ID`] (I2).
    WrongTxids(u64),
    /// Blok `h` belum ada walaupun ujungnya di atas `h`.
    Missing(u64),
}

/// Sumber data palsu di atas [`Chain`].
pub(crate) struct FakeSource<'a> {
    pub(crate) chain: &'a Chain,
    pub(crate) mempool: Vec<Hash32>,
    pub(crate) fault: Fault,
}

impl FakeSource<'_> {
    fn online(&self) -> Result<(), SourceError> {
        match self.fault {
            Fault::Offline => Err(SourceError("node tidak bisa dihubungi".to_owned())),
            _ => Ok(()),
        }
    }
}

impl ChainSource for FakeSource<'_> {
    fn tip(&self) -> Result<(u64, Hash32), SourceError> {
        self.online()?;
        let height = self.chain.height();
        Ok((height, self.chain.id(height)))
    }

    fn block_id_at(&self, height: u64) -> Result<Option<Hash32>, SourceError> {
        self.online()?;
        match self.fault {
            Fault::WrongId(bad) if bad == height => return Ok(Some(FAKE_ID)),
            Fault::Missing(gone) if gone == height => return Ok(None),
            _ => {}
        }
        let index = usize::try_from(height).unwrap();
        Ok(self.chain.blocks.get(index).map(id_of))
    }

    fn block(&self, id: &Hash32) -> Result<Option<SourceBlock>, SourceError> {
        self.online()?;
        let found = match self.fault {
            Fault::WrongId(bad) if *id == FAKE_ID => {
                self.chain.blocks.get(usize::try_from(bad).unwrap())
            }
            _ => self.chain.blocks.iter().find(|block| id_of(block) == *id),
        };
        Ok(found.map(|block| {
            let mut txids: Vec<Hash32> = block.txs().iter().map(txid).collect();
            if let Fault::WrongTxids(bad) = self.fault
                && block.header().height().get() == bad
                && let Some(first) = txids.first_mut()
            {
                *first = FAKE_ID;
            }
            SourceBlock {
                bytes: block.to_bytes(),
                txids,
            }
        }))
    }

    fn mempool(&self) -> Result<Vec<Hash32>, SourceError> {
        self.online()?;
        Ok(self.mempool.clone())
    }
}

/// Node tiruan di atas [`Chain`]: hanya `get_chain_info`, `get_block_hash`, `get_block`, dan `get_mempool` yang
/// berisi. Chainwork = tinggi + 1 (byte terakhir).
pub(crate) struct FakeNode {
    chain: Mutex<Chain>,
    mempool: Mutex<Vec<Hash32>>,
}

impl FakeNode {
    pub(crate) fn new(chain: Chain) -> Self {
        Self {
            chain: Mutex::new(chain),
            mempool: Mutex::new(Vec::new()),
        }
    }

    pub(crate) fn set_chain(&self, chain: Chain) {
        *self.chain.lock().unwrap() = chain;
    }

    pub(crate) fn set_mempool(&self, txids: Vec<Hash32>) {
        *self.mempool.lock().unwrap() = txids;
    }
}

impl RpcBackend for FakeNode {
    fn chain_info(&self) -> Result<ChainInfo, RpcError> {
        let chain = self.chain.lock().unwrap();
        let height = chain.height();
        let mut chainwork = [0_u8; 32];
        chainwork[24..].copy_from_slice(&(height + 1).to_be_bytes());
        Ok(ChainInfo {
            network: NetworkName::new(Network::Regtest),
            height,
            best_block_id: Hex32::from(chain.id(height)),
            chainwork: Hex32::new(chainwork),
            next_target: CompactHex::new(LIMIT),
            median_time_past: T0,
            mined_total: Quanta::new(chain.mined_total()),
            total_supply: Quanta::new(total_supply_after(chain.mined_total()).unwrap()),
            mempool_count: u64::try_from(self.mempool.lock().unwrap().len()).unwrap(),
        })
    }

    fn block_hash(&self, height: u64) -> Result<Hex32, RpcError> {
        let chain = self.chain.lock().unwrap();
        let index = usize::try_from(height).unwrap();
        chain
            .blocks
            .get(index)
            .map(|block| Hex32::from(id_of(block)))
            .ok_or(RpcError::NotFound)
    }

    fn block(&self, block_id: Hex32) -> Result<BlockInfo, RpcError> {
        let chain = self.chain.lock().unwrap();
        let id = block_id.to_hash32();
        let block = chain
            .blocks
            .iter()
            .find(|block| id_of(block) == id)
            .ok_or(RpcError::NotFound)?;
        let txids: Vec<Hash32> = block.txs().iter().map(txid).collect();
        Ok(BlockInfo::from_block(block, id, &txids))
    }

    fn block_template(&self, _params: &GetBlockTemplateParams) -> Result<BlockTemplate, RpcError> {
        Err(RpcError::NotFound)
    }

    fn submit_block(&self, _block: &[u8]) -> Result<SubmitBlockResult, RpcError> {
        Err(RpcError::NotFound)
    }

    fn send_raw_transaction(&self, _tx: &[u8]) -> Result<SendRawTransactionResult, RpcError> {
        Err(RpcError::InvalidEncoding)
    }

    fn mempool(&self) -> Result<MempoolInfo, RpcError> {
        let txids = self
            .mempool
            .lock()
            .unwrap()
            .iter()
            .copied()
            .map(Hex32::from)
            .collect();
        Ok(MempoolInfo { txids })
    }

    fn scan_utxos(&self, _payloads: &[Hex32]) -> Result<Vec<UtxoInfo>, RpcError> {
        Ok(Vec::new())
    }

    fn scan_mempool(&self, _payloads: &[Hex32]) -> Result<MempoolScan, RpcError> {
        Ok(MempoolScan::default())
    }
}
