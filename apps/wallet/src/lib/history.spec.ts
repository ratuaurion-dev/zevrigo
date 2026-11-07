import { describe, expect, test } from 'vitest';
import { type HistoryEntryDto, classify, confirmations, filterEntries } from './history';

function entry(over: Partial<HistoryEntryDto>): HistoryEntryDto {
  return {
    txid: 'ab'.repeat(32),
    height: 10,
    timestamp: 1_788_000_000,
    coinbase: false,
    received: '0',
    spent: '0',
    fee: null,
    ...over,
  };
}

describe('riwayat', () => {
  test('menggolongkan hadiah tambang, masuk, keluar, dan ke diri sendiri', () => {
    expect(classify(entry({ coinbase: true, received: '4361021521773785' }))).toEqual({
      direction: 'mined',
      net: 4_361_021_521_773_785n,
    });
    expect(classify(entry({ received: '500' }))).toEqual({ direction: 'incoming', net: 500n });
    expect(classify(entry({ received: '40', spent: '100', fee: '10' }))).toEqual({ direction: 'outgoing', net: -60n });
    expect(classify(entry({ received: '100', spent: '100' }))).toEqual({ direction: 'self', net: 0n });
  });

  test('menghitung konfirmasi dan menyaring entri', () => {
    expect(confirmations(10, 10)).toBe(1);
    expect(confirmations(1, 100)).toBe(100);
    expect(confirmations(null, 100)).toBe(0);
    expect(confirmations(11, 10)).toBe(0);
    const entries = [
      entry({ txid: `aa${'0'.repeat(62)}`, coinbase: true, received: '5' }),
      entry({ txid: `bb${'0'.repeat(62)}`, received: '7' }),
      entry({ txid: `cc${'0'.repeat(62)}`, received: '1', spent: '9', height: null }),
    ];
    expect(filterEntries(entries, 'all', '').length).toBe(3);
    expect(filterEntries(entries, 'incoming', '').map((e) => e.received)).toEqual(['7']);
    expect(filterEntries(entries, 'outgoing', '').map((e) => e.spent)).toEqual(['9']);
    expect(filterEntries(entries, 'all', ' BB ').map((e) => e.received)).toEqual(['7']);
    expect(filterEntries(entries, 'mined', 'cc')).toEqual([]);
  });
});
