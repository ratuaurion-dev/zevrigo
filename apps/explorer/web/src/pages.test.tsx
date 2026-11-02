import { renderToStaticMarkup } from 'react-dom/server';
import { describe, expect, it } from 'vitest';
import type { AddressInfo, AllocationInfo, BlockDetail, MempoolInfo, Status, SupplyInfo, TxDetail } from './api';
import { dict } from './i18n';
import type { Lang } from './i18n';
import { AddressView } from './pages/address';
import { AllocationView } from './pages/allocation';
import { BlockView } from './pages/block';
import { BlocksView } from './pages/blocks';
import { StatusPanel } from './pages/home';
import { MempoolView } from './pages/mempool';
import { SupplyView } from './pages/supply';
import { TxView } from './pages/tx';
import type { Ui } from './ui';

const H = (c: string) => c.repeat(64);
const ALLOCATION = '4380000000000000000000000';
const ADDR_GENESIS = 'rhsis1qqqsyqcyq5rqwzqfpg9scrgwpugpzysnzs23v9ccrydpk8qarc0jqszjs4m';
const ADDR_RECEIVER = 'rhsis1zzzsyqcyq5rqwzqfpg9scrgwpugpzysnzs23v9ccrydpk8qarc0jqszjs4m';

function ui(lang: Lang): Ui {
  return { t: dict(lang), lang };
}

const block: BlockDetail = {
  block_id: H('b'),
  height: 100,
  prev_block_id: H('a'),
  merkle_root: H('c'),
  timestamp: 1_700_000_000,
  target: '207fffff',
  nonce: '42',
  size: 1234,
  tx_count: 2,
  subsidy: '1910127426536917000',
  fees: '1000',
  confirmations: 3,
  txids: [H('d'), H('e')],
  next_block_id: null,
};

const tx: TxDetail = {
  txid: H('e'),
  block_id: H('b'),
  height: 100,
  position: 1,
  size: 284,
  is_coinbase: false,
  fee: '1000',
  confirmations: 7,
  inputs: [{ txid: H('f'), index: 0, amount: ALLOCATION, kind: 0, payload: H('1'), address: ADDR_GENESIS }],
  outputs: [
    { index: 0, amount: '4379999999999999999999000', kind: 0, payload: H('2'), address: ADDR_RECEIVER, spent_by: null },
  ],
};

function address(page: number, txCount: number): AddressInfo {
  return {
    address: ADDR_RECEIVER,
    kind: 0,
    payload: H('2'),
    balance: '5000000000000000',
    received: '5000000000000000',
    sent: '0',
    utxo_count: 1,
    tx_count: txCount,
    page,
    page_size: 25,
    txs: [{ txid: H('e'), height: 100, confirmations: 1, received: '5000000000000000', sent: '0' }],
  };
}

const supply: SupplyInfo = {
  height: 105,
  mined_total: '1000000000000000000000',
  scheduled_mined_total: '1000000000000000000000',
  matches_schedule: true,
  genesis_allocation: ALLOCATION,
  total_supply: '4381000000000000000000000',
  fees_total: '1000',
  next_subsidy: '2000000000000000000',
  tip_target: '207fffff',
  average_block_seconds: 120,
};

describe('pages', () => {
  it('renders block view', () => {
    const html = renderToStaticMarkup(<BlockView block={block} ui={ui('id')} />);
    expect(html).toContain('Blok 100');
    expect(html).toContain('3 konfirmasi · belum aman');
    expect(html).toContain('2023-11-14 22:13:20 UTC');
    expect(html).toContain('1,910127426536917 HSIS');
    expect(html).toContain('0,000000000000001 HSIS');
    expect(html).toContain(`href="#/block/${H('a')}"`);
    expect(html).toContain(`href="#/tx/${H('d')}"`);
    expect(html).toContain('(coinbase)');
    expect(html).toContain('1.234');
    expect(html).not.toContain('undefined');
    expect(html).not.toContain('NaN');
  });

  it('renders transaction view', () => {
    const html = renderToStaticMarkup(<TxView tx={tx} ui={ui('id')} />);
    expect(html).toContain('7 konfirmasi');
    expect(html).not.toContain('belum aman');
    expect(html).toContain('4.380.000 HSIS');
    expect(html).toContain('4.379.999,999999999999999 HSIS');
    expect(html).toContain(`title="${ALLOCATION} quanta"`);
    expect(html).toContain(`href="#/address/${ADDR_GENESIS}"`);
    expect(html).toContain('Belum dibelanjakan');
    const coinbase = renderToStaticMarkup(<TxView tx={{ ...tx, is_coinbase: true, inputs: [] }} ui={ui('id')} />);
    expect(coinbase).toContain('Coinbase (hadiah tambang, tanpa input)');
  });

  it('renders address view with pager', () => {
    const first = renderToStaticMarkup(<AddressView info={address(0, 60)} ui={ui('id')} />);
    expect(first).toContain('0,005 HSIS');
    expect(first).toContain('Halaman 1');
    expect(first).toContain(`href="#/address/${ADDR_RECEIVER}?page=1"`);
    expect(first).not.toContain('Sebelumnya');
    const last = renderToStaticMarkup(<AddressView info={address(2, 60)} ui={ui('id')} />);
    expect(last).toContain(`href="#/address/${ADDR_RECEIVER}?page=1"`);
    expect(last).not.toContain('Berikutnya');
  });

  it('renders supply and allocation views', () => {
    const html = renderToStaticMarkup(<SupplyView supply={supply} ui={ui('id')} />);
    expect(html).toContain('1.000 HSIS');
    expect(html).toContain('4.381.000 HSIS');
    expect(html).toContain('120 detik');
    expect(html).not.toContain('tidak sesuai');
    const bad = renderToStaticMarkup(<SupplyView supply={{ ...supply, matches_schedule: false }} ui={ui('id')} />);
    expect(bad).toContain('Pasokan tidak sesuai jadwal rumus');
    const allocation: AllocationInfo = {
      genesis_block_id: H('0'),
      allocated: ALLOCATION,
      unspent: '0',
      outputs: [
        { txid: H('f'), index: 0, amount: ALLOCATION, kind: 0, payload: H('1'), address: ADDR_GENESIS, spent_by: { txid: H('e'), input: 0, height: 100 } },
      ],
      addresses: [{ ...address(0, 2), address: ADDR_GENESIS, balance: '0', received: ALLOCATION, sent: ALLOCATION }],
    };
    const page = renderToStaticMarkup(<AllocationView info={allocation} ui={ui('id')} />);
    expect(page).toContain('Alokasi pencipta (genesis)');
    expect(page).toContain(`href="#/block/${H('0')}"`);
    expect(page).toContain(`href="#/tx/${H('e')}"`);
    expect(page).toContain('4.380.000 HSIS');
    expect(page).toContain(ADDR_GENESIS);
  });

  it('renders blocks list, status and mempool', () => {
    const rows = [
      { height: 21, block_id: H('3'), timestamp: 0, size: 300, tx_count: 1, confirmations: 1 },
      { height: 20, block_id: H('4'), timestamp: 0, size: 300, tx_count: 1, confirmations: 2 },
    ];
    const list = renderToStaticMarkup(<BlocksView rows={rows} before="22" ui={ui('id')} />);
    expect(list).toContain('href="#/blocks?before=20"');
    expect(list).toContain('href="#/blocks"');
    expect(list).toContain('1970-01-01 00:00:00 UTC');
    const status: Status = {
      network: 'regtest',
      indexed_height: 105,
      indexed_block_id: H('5'),
      node_height: 1200,
      node_best_block_id: H('6'),
      chainwork: H('0'),
      mempool_count: 0,
      index_error: null,
      last_error: 'node tidak bisa dihubungi',
    };
    const panel = renderToStaticMarkup(<StatusPanel status={status} ui={ui('id')} />);
    expect(panel).toContain('regtest');
    expect(panel).toContain('1.200');
    expect(panel).toContain('Sinkronisasi terakhir dengan node gagal: node tidak bisa dihubungi');
    const mempool: MempoolInfo = { count: 1, txids: [H('7')] };
    const pool = renderToStaticMarkup(<MempoolView mempool={mempool} ui={ui('id')} />);
    expect(pool).toContain(`href="#/tx/${H('7')}"`);
    expect(renderToStaticMarkup(<MempoolView mempool={{ count: 0, txids: [] }} ui={ui('id')} />)).toContain('Mempool kosong.');
  });

  it('renders English labels and number format', () => {
    const html = renderToStaticMarkup(<TxView tx={tx} ui={ui('en')} />);
    expect(html).toContain('7 confirmations');
    expect(html).toContain('4,380,000 HSIS');
    expect(html).toContain('4,379,999.999999999999999 HSIS');
    expect(html).toContain('Unspent');
    const blockHtml = renderToStaticMarkup(<BlockView block={block} ui={ui('en')} />);
    expect(blockHtml).toContain('Block 100');
    expect(blockHtml).toContain('3 confirmations · not yet safe');
    expect(renderToStaticMarkup(<SupplyView supply={supply} ui={ui('en')} />)).toContain('120 seconds');
  });
});
