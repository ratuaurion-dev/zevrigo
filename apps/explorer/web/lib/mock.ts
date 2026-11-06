import type {
  BlockDetail,
  BlockList,
  BlockRow,
  MempoolInfo,
  SearchResult,
  Status,
  SupplyInfo,
  TxDetail,
} from './api';

/*
 * Bentuk data lengkap di dalam data tiruan saja. Fungsi `getMock*` hanya mengeluarkan bentuk publik API (DEC-28):
 * tanpa alamat, jumlah per output, jejak koin, maupun alokasi genesis.
 */
type SpentBy = { txid: string; input: number; height: number };
type TxInput = { txid: string; index: number; amount: string; kind: number; payload: string; address: string };
type TxOutput = {
  index: number;
  amount: string;
  kind: number;
  payload: string;
  address: string;
  spent_by: SpentBy | null;
};
type FullTx = Omit<TxDetail, 'input_count' | 'output_count'> & { inputs: TxInput[]; outputs: TxOutput[] };
type AddressTx = { txid: string; height: number; confirmations: number; received: string; sent: string };
type AddressInfo = {
  address: string;
  kind: number;
  payload: string;
  balance: string;
  received: string;
  sent: string;
  utxo_count: number;
  tx_count: number;
  page: number;
  page_size: number;
  txs: AddressTx[];
};
type AllocationInfo = {
  genesis_block_id: string;
  allocated: string;
  unspent: string;
  outputs: (TxOutput & { txid: string })[];
  addresses: AddressInfo[];
};
type FullSupply = SupplyInfo & { genesis_allocation: string; total_supply: string };

function toPublicTx(full: FullTx): TxDetail {
  const { inputs, outputs, ...rest } = full;
  return { ...rest, input_count: inputs.length, output_count: outputs.length };
}

/**
 * Deterministic 64-character lowercase hex generator from a tag + integer seed.
 */
function makeHex64(prefixTag: string, index: number): string {
  const hexTag = prefixTag
    .split('')
    .map((c) => c.charCodeAt(0).toString(16).padStart(2, '0'))
    .join('');
  const idxHex = index.toString(16).padStart(8, '0');
  // Create deterministic 64-char hex string using simple LCG mixing
  let state = (index * 2654435761 + 0x9e3779b9) >>> 0;
  let fill = '';
  while (fill.length < 64) {
    state = (Math.imul(state ^ (state >>> 15), 2246822507) + 0x85ebca6b) >>> 0;
    fill += state.toString(16).padStart(8, '0');
  }
  const combined = (hexTag + idxHex + fill).toLowerCase().replace(/[^0-9a-f]/g, 'a');
  return combined.slice(0, 64);
}

/**
 * Deterministic Regtest bech32m address generator (`rhsis1...`) using only valid bech32 charset:
 * `qpzry9x8gf2tvdw0s3jn54khce6mua7l`
 */
const BECH32_CHARSET = 'qpzry9x8gf2tvdw0s3jn54khce6mua7l';
function makeRegtestAddress(seed: number): string {
  let state = (seed * 1664525 + 1013904223) >>> 0;
  let body = '';
  for (let i = 0; i < 38; i++) {
    state = (Math.imul(state, 22695477) + 1) >>> 0;
    body += BECH32_CHARSET[state % BECH32_CHARSET.length];
  }
  return `rhsis1${body}`;
}

export const CHAIN_TIP_HEIGHT = 120;
// Base genesis timestamp: 2026-10-31 00:00:00 UTC (1793404800)
export const GENESIS_TIMESTAMP = 1_793_404_800;

// Exact required protocol constants & subsidies
export const GENESIS_ALLOCATION_STR = '4380000000000000000000000'; // 4,380,000 ZVRG
export const SUBSIDY_HEIGHT_1 = '1910127426536917';
export const SUBSIDY_HEIGHT_2 = '3820254852907233';
export const SUBSIDY_HEIGHT_3 = '5730382278861043';

const S_MINING_QUANTA = 43_800_000n * 1_000_000_000_000_000_000n;
const ONE_ZVRG_QUANTA = 1_000_000_000_000_000_000n;
const EMISSION_DIVISOR = 2_274_842n;
const SLOW_START_BLOCKS = 10_080n;

// Key addresses
export const CREATOR_PRIMARY_ADDRESS = 'rhsis1qpzry9x8gf2tvdw0s3jn54khce6mua7lqpzry9';
export const CREATOR_RESERVE_ADDRESS = 'rhsis1x8gf2tvdw0s3jn54khce6mua7lqpzry9x8gf2t';
export const RECIPIENT_TRANSFER_ADDRESS = 'rhsis1tvdw0s3jn54khce6mua7lqpzry9x8gf2tvdw0s';
export const MINER_POOL_ADDRESS = 'rhsis1s3jn54khce6mua7lqpzry9x8gf2tvdw0s3jn54';

// Key mempool txids (including one highlighted pending txid for easy testing)
export const MEMPOOL_ONLY_TXID =
  'e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5';
export const MEMPOOL_TXIDS: string[] = [
  MEMPOOL_ONLY_TXID,
  'a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2',
  'f9e8d7c6b5a4f3e2d1c0b9a8f7e6d5c4b3a2f1e0d9c8b7a6f5e4d3c2b1a0f9e8',
];

interface BuiltChain {
  blocksByHeight: Map<number, BlockDetail>;
  blocksByHash: Map<string, BlockDetail>;
  txsByHash: Map<string, FullTx>;
  addressesByBech32: Map<string, AddressInfo>;
  supply: FullSupply;
  allocation: AllocationInfo;
}

let cachedChain: BuiltChain | null = null;

function buildMockChain(): BuiltChain {
  if (cachedChain) return cachedChain;

  const blocksByHeight = new Map<number, BlockDetail>();
  const blocksByHash = new Map<string, BlockDetail>();
  const txsByHash = new Map<string, FullTx>();

  // Precompute block hashes 0..120
  const blockHashes: string[] = [];
  for (let h = 0; h <= CHAIN_TIP_HEIGHT; h++) {
    blockHashes.push(makeHex64('b10c', h));
  }

  let minedSoFar = 0n;
  let cumulativeFees = 0n;
  let currentTimestamp = GENESIS_TIMESTAMP;

  // Genesis allocation split across two outputs in block #0:
  // Output 0: 3,000,000 ZVRG to CREATOR_PRIMARY_ADDRESS (spent at height 100)
  // Output 1: 1,380,000 ZVRG to CREATOR_RESERVE_ADDRESS (still unspent)
  const genesisOut0Amount = (3_000_000n * ONE_ZVRG_QUANTA).toString(10);
  const genesisOut1Amount = (1_380_000n * ONE_ZVRG_QUANTA).toString(10);
  const genesisTxid = makeHex64('c01b', 0);
  const height100SpendTxid = makeHex64('fd00', 100);

  // Height 100 spend amounts:
  // Spends Output 0 (3,000,000 ZVRG) with fee = "1000" quanta
  // Sends 500,000 ZVRG to RECIPIENT_TRANSFER_ADDRESS
  // Returns 2,500,000 ZVRG - 1,000 quanta change to CREATOR_PRIMARY_ADDRESS
  const transferAmount = 500_000n * ONE_ZVRG_QUANTA;
  const feeHeight100 = 1000n;
  const changeAmount = 3_000_000n * ONE_ZVRG_QUANTA - transferAmount - feeHeight100;

  for (let h = 0; h <= CHAIN_TIP_HEIGHT; h++) {
    if (h > 0) {
      // Vary interval slightly around 120s (112s to 128s) while averaging 120s
      const delta = 120 + ((h % 5) - 2) * 3;
      currentTimestamp += delta;
    }

    // Compute exact block subsidy according to protocol schedule
    let subsidyQuanta = 0n;
    if (h === 0) {
      subsidyQuanta = 0n;
    } else if (h === 1) {
      subsidyQuanta = BigInt(SUBSIDY_HEIGHT_1);
    } else if (h === 2) {
      subsidyQuanta = BigInt(SUBSIDY_HEIGHT_2);
    } else if (h === 3) {
      subsidyQuanta = BigInt(SUBSIDY_HEIGHT_3);
    } else {
      const baseReward = (S_MINING_QUANTA - minedSoFar) / EMISSION_DIVISOR;
      const clampedReward = baseReward > ONE_ZVRG_QUANTA ? baseReward : ONE_ZVRG_QUANTA;
      subsidyQuanta = (clampedReward * BigInt(h)) / SLOW_START_BLOCKS;
    }

    const confirmations = CHAIN_TIP_HEIGHT - h + 1;
    const blockId = blockHashes[h];
    const prevBlockId = h === 0 ? '0'.repeat(64) : blockHashes[h - 1];
    const nextBlockId = h < CHAIN_TIP_HEIGHT ? blockHashes[h + 1] : null;
    const coinbaseTxid = h === 0 ? genesisTxid : makeHex64('c01b', h);

    const hasRegularTx = h === 100;
    const blockFees = hasRegularTx ? feeHeight100 : 0n;
    const txids = hasRegularTx ? [coinbaseTxid, height100SpendTxid] : [coinbaseTxid];
    const blockSize = h === 0 ? 384 : hasRegularTx ? 642 : 240 + (h % 9) * 12;

    if (h === 0) {
      // Genesis coinbase transaction containing the 4,380,000 ZVRG creator allocation
      const genesisTx: FullTx = {
        txid: genesisTxid,
        block_id: blockId,
        height: 0,
        position: 0,
        size: 312,
        is_coinbase: true,
        fee: '0',
        confirmations,
        inputs: [],
        outputs: [
          {
            index: 0,
            amount: genesisOut0Amount,
            kind: 0,
            payload: makeHex64('a110', 1),
            address: CREATOR_PRIMARY_ADDRESS,
            spent_by: {
              txid: height100SpendTxid,
              input: 0,
              height: 100,
            },
          },
          {
            index: 1,
            amount: genesisOut1Amount,
            kind: 0,
            payload: makeHex64('a110', 2),
            address: CREATOR_RESERVE_ADDRESS,
            spent_by: null,
          },
        ],
      };
      txsByHash.set(genesisTxid, genesisTx);
    } else {
      const minerReward = subsidyQuanta + blockFees;
      const minerAddress = h % 3 === 0 ? MINER_POOL_ADDRESS : makeRegtestAddress(h);
      const coinbaseTx: FullTx = {
        txid: coinbaseTxid,
        block_id: blockId,
        height: h,
        position: 0,
        size: 185,
        is_coinbase: true,
        fee: '0',
        confirmations,
        inputs: [],
        outputs: [
          {
            index: 0,
            amount: minerReward.toString(10),
            kind: 0,
            payload: makeHex64('cb00', h),
            address: minerAddress,
            spent_by: null,
          },
        ],
      };
      txsByHash.set(coinbaseTxid, coinbaseTx);

      if (hasRegularTx) {
        const spendTx: FullTx = {
          txid: height100SpendTxid,
          block_id: blockId,
          height: 100,
          position: 1,
          size: 385,
          is_coinbase: false,
          fee: feeHeight100.toString(10),
          confirmations,
          inputs: [
            {
              txid: genesisTxid,
              index: 0,
              amount: genesisOut0Amount,
              kind: 0,
              payload: makeHex64('a110', 1),
              address: CREATOR_PRIMARY_ADDRESS,
            },
          ],
          outputs: [
            {
              index: 0,
              amount: transferAmount.toString(10),
              kind: 0,
              payload: makeHex64('a110', 3),
              address: RECIPIENT_TRANSFER_ADDRESS,
              spent_by: null,
            },
            {
              index: 1,
              amount: changeAmount.toString(10),
              kind: 0,
              payload: makeHex64('a110', 1),
              address: CREATOR_PRIMARY_ADDRESS,
              spent_by: null,
            },
          ],
        };
        txsByHash.set(height100SpendTxid, spendTx);
      }
    }

    minedSoFar += subsidyQuanta;
    cumulativeFees += blockFees;

    const blockDetail: BlockDetail = {
      block_id: blockId,
      height: h,
      prev_block_id: prevBlockId,
      merkle_root: makeHex64('eb00', h),
      timestamp: currentTimestamp,
      target: '207fffff',
      nonce: String(104_857 + h * 7_919),
      size: blockSize,
      tx_count: txids.length,
      subsidy: subsidyQuanta.toString(10),
      fees: blockFees.toString(10),
      confirmations,
      txids,
      next_block_id: nextBlockId,
    };

    blocksByHeight.set(h, blockDetail);
    blocksByHash.set(blockId, blockDetail);
  }

  // Compute next subsidy at height 121
  const nextHeight = BigInt(CHAIN_TIP_HEIGHT + 1);
  const nextBaseReward = (S_MINING_QUANTA - minedSoFar) / EMISSION_DIVISOR;
  const nextClamped = nextBaseReward > ONE_ZVRG_QUANTA ? nextBaseReward : ONE_ZVRG_QUANTA;
  const nextSubsidy = (nextClamped * nextHeight) / SLOW_START_BLOCKS;
  const genesisAllocationQuanta = BigInt(GENESIS_ALLOCATION_STR);
  const totalSupplyQuanta = minedSoFar + genesisAllocationQuanta;

  const supply: FullSupply = {
    height: CHAIN_TIP_HEIGHT,
    mined_total: minedSoFar.toString(10),
    scheduled_mined_total: minedSoFar.toString(10),
    matches_schedule: true,
    genesis_allocation: GENESIS_ALLOCATION_STR,
    total_supply: totalSupplyQuanta.toString(10),
    fees_total: cumulativeFees.toString(10),
    next_subsidy: nextSubsidy.toString(10),
    tip_target: '207fffff',
    average_block_seconds: 120,
  };

  // Build AddressInfo records for all addresses appearing in transactions
  const addressesByBech32 = new Map<string, AddressInfo>();
  const addressTxsMap = new Map<
    string,
    {
      kind: number;
      payload: string;
      received: bigint;
      sent: bigint;
      utxoCount: number;
      txs: AddressTx[];
    }
  >();

  // Traverse heights newest first so address tx history is newest first
  for (let h = CHAIN_TIP_HEIGHT; h >= 0; h--) {
    const blk = blocksByHeight.get(h)!;
    for (const txid of blk.txids) {
      const tx = txsByHash.get(txid)!;
      const perAddress = new Map<string, { received: bigint; sent: bigint; kind: number; payload: string }>();

      for (const inp of tx.inputs) {
        const cur = perAddress.get(inp.address) ?? {
          received: 0n,
          sent: 0n,
          kind: inp.kind,
          payload: inp.payload,
        };
        cur.sent += BigInt(inp.amount);
        perAddress.set(inp.address, cur);
      }

      for (const out of tx.outputs) {
        const cur = perAddress.get(out.address) ?? {
          received: 0n,
          sent: 0n,
          kind: out.kind,
          payload: out.payload,
        };
        cur.received += BigInt(out.amount);
        perAddress.set(out.address, cur);

        if (out.spent_by === null) {
          const agg = addressTxsMap.get(out.address) ?? {
            kind: out.kind,
            payload: out.payload,
            received: 0n,
            sent: 0n,
            utxoCount: 0,
            txs: [],
          };
          agg.utxoCount += 1;
          addressTxsMap.set(out.address, agg);
        }
      }

      for (const [addr, delta] of perAddress.entries()) {
        const agg = addressTxsMap.get(addr) ?? {
          kind: delta.kind,
          payload: delta.payload,
          received: 0n,
          sent: 0n,
          utxoCount: 0,
          txs: [],
        };
        agg.received += delta.received;
        agg.sent += delta.sent;
        agg.txs.push({
          txid: tx.txid,
          height: tx.height,
          confirmations: tx.confirmations,
          received: delta.received.toString(10),
          sent: delta.sent.toString(10),
        });
        addressTxsMap.set(addr, agg);
      }
    }
  }

  for (const [addr, agg] of addressTxsMap.entries()) {
    const balance = agg.received >= agg.sent ? agg.received - agg.sent : 0n;
    addressesByBech32.set(addr, {
      address: addr,
      kind: agg.kind,
      payload: agg.payload,
      balance: balance.toString(10),
      received: agg.received.toString(10),
      sent: agg.sent.toString(10),
      utxo_count: agg.utxoCount,
      tx_count: agg.txs.length,
      page: 0,
      page_size: 25,
      txs: agg.txs,
    });
  }

  const genesisTx = txsByHash.get(genesisTxid)!;
  const allocation: AllocationInfo = {
    genesis_block_id: blockHashes[0],
    allocated: GENESIS_ALLOCATION_STR,
    unspent: genesisOut1Amount, // Output 1 (1,380,000 ZVRG) remains unspent in genesis outputs
    outputs: genesisTx.outputs.map((out) => ({
      ...out,
      txid: genesisTxid,
    })),
    addresses: [
      addressesByBech32.get(CREATOR_PRIMARY_ADDRESS)!,
      addressesByBech32.get(CREATOR_RESERVE_ADDRESS)!,
    ],
  };

  cachedChain = {
    blocksByHeight,
    blocksByHash,
    txsByHash,
    addressesByBech32,
    supply,
    allocation,
  };
  return cachedChain;
}

export type MockModeOverride = 'normal' | 'notready' | 'indexerror' | 'offline';

export function getActiveMockOverride(): MockModeOverride {
  if (typeof window === 'undefined') return 'normal';
  try {
    const params = new URLSearchParams(window.location.search);
    const m = params.get('mock');
    if (m === 'notready' || m === 'indexerror' || m === 'offline') {
      return m;
    }
  } catch {
    // ignore
  }
  return 'normal';
}

export function getMockStatus(override = getActiveMockOverride()): Status {
  const chain = buildMockChain();
  const tipBlock = chain.blocksByHeight.get(CHAIN_TIP_HEIGHT)!;

  if (override === 'notready') {
    return {
      network: 'regtest',
      indexed_height: null,
      indexed_block_id: null,
      node_height: CHAIN_TIP_HEIGHT,
      node_best_block_id: tipBlock.block_id,
      chainwork: '00000000000000000000000000000000000000000000000000000000000001e2',
      mempool_count: MEMPOOL_TXIDS.length,
      index_error: null,
      last_error: null,
    };
  }

  if (override === 'indexerror') {
    return {
      network: 'regtest',
      indexed_height: CHAIN_TIP_HEIGHT,
      indexed_block_id: tipBlock.block_id,
      node_height: CHAIN_TIP_HEIGHT + 1,
      node_best_block_id: makeHex64('b10c', CHAIN_TIP_HEIGHT + 1),
      chainwork: '00000000000000000000000000000000000000000000000000000000000001e2',
      mempool_count: MEMPOOL_TXIDS.length,
      index_error:
        'block 121 coinbase subsidy 99000000000000000000 exceeds scheduled maximum 231125418610966957',
      last_error: null,
    };
  }

  return {
    network: 'regtest',
    indexed_height: CHAIN_TIP_HEIGHT,
    indexed_block_id: tipBlock.block_id,
    node_height: CHAIN_TIP_HEIGHT,
    node_best_block_id: tipBlock.block_id,
    chainwork: '00000000000000000000000000000000000000000000000000000000000001e2',
    mempool_count: MEMPOOL_TXIDS.length,
    index_error: null,
    last_error: null,
  };
}

export function getMockBlocks(beforeHeight?: number): BlockList {
  const chain = buildMockChain();
  const startHeight =
    beforeHeight !== undefined && beforeHeight <= CHAIN_TIP_HEIGHT
      ? beforeHeight - 1
      : CHAIN_TIP_HEIGHT;

  const rows: BlockRow[] = [];
  for (let h = startHeight; h >= 0 && rows.length < 20; h--) {
    const b = chain.blocksByHeight.get(h);
    if (b) {
      rows.push({
        height: b.height,
        block_id: b.block_id,
        timestamp: b.timestamp,
        size: b.size,
        tx_count: b.tx_count,
        confirmations: b.confirmations,
      });
    }
  }
  return { blocks: rows };
}

export function getMockBlockDetail(heightOrHash: string): BlockDetail | null {
  const chain = buildMockChain();
  if (/^(0|[1-9][0-9]{0,19})$/.test(heightOrHash)) {
    const h = Number(heightOrHash);
    return chain.blocksByHeight.get(h) ?? null;
  }
  return chain.blocksByHash.get(heightOrHash.toLowerCase()) ?? null;
}

export function getMockTxDetail(txid: string): {
  tx: TxDetail | null;
  inMempool: boolean;
} {
  const chain = buildMockChain();
  const norm = txid.toLowerCase();
  const found = chain.txsByHash.get(norm) ?? null;
  if (found) {
    return { tx: toPublicTx(found), inMempool: false };
  }
  if (MEMPOOL_TXIDS.includes(norm)) {
    return { tx: null, inMempool: true };
  }
  return { tx: null, inMempool: false };
}

export function getMockSupplyInfo(): SupplyInfo {
  const { genesis_allocation: _genesis, total_supply: _total, ...supply } = buildMockChain().supply;
  return supply;
}

export function getMockMempoolInfo(): MempoolInfo {
  return { count: MEMPOOL_TXIDS.length };
}

export function getMockSearchResult(query: string): {
  result: SearchResult | null;
  inMempool: boolean;
} {
  const chain = buildMockChain();
  const q = query.trim().toLowerCase();

  if (/^(0|[1-9][0-9]{0,19})$/.test(q)) {
    const h = Number(q);
    const blk = chain.blocksByHeight.get(h);
    if (blk) {
      return {
        result: { kind: 'block', key: blk.block_id },
        inMempool: false,
      };
    }
    return { result: null, inMempool: false };
  }

  if (/^[0-9a-f]{64}$/.test(q)) {
    const blk = chain.blocksByHash.get(q);
    if (blk) {
      return {
        result: { kind: 'block', key: blk.block_id },
        inMempool: false,
      };
    }
    const tx = chain.txsByHash.get(q);
    if (tx) {
      return {
        result: { kind: 'tx', key: tx.txid },
        inMempool: false,
      };
    }
    if (MEMPOOL_TXIDS.includes(q)) {
      return {
        result: null,
        inMempool: true,
      };
    }
    return { result: null, inMempool: false };
  }

  return { result: null, inMempool: false };
}

export function getSampleShowcaseIds() {
  const chain = buildMockChain();
  const tipBlock = chain.blocksByHeight.get(CHAIN_TIP_HEIGHT)!;
  const block100 = chain.blocksByHeight.get(100)!;
  const genesisBlock = chain.blocksByHeight.get(0)!;
  return {
    tipBlockId: tipBlock.block_id,
    genesisBlockId: genesisBlock.block_id,
    spendTxId: block100.txids[1],
    genesisCoinbaseTxId: genesisBlock.txids[0],
    mempoolPendingTxId: MEMPOOL_ONLY_TXID,
  };
}
