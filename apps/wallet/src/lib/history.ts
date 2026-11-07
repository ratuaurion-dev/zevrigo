/**
 * Entri riwayat dari wallet (Rust `HistoryEntryView`) dan penggolongannya untuk tampilan. Jumlah tetap teks quanta.
 */

import { toQuanta } from './quanta';

export interface HistoryEntryDto {
  txid: string;
  height: number | null;
  timestamp: number | null;
  coinbase: boolean;
  received: string;
  spent: string;
  fee: string | null;
}

export type Direction = 'mined' | 'incoming' | 'outgoing' | 'self';

/**
 * Jenis entri dan perubahan saldo bersih (`received - spent`): hadiah tambang, masuk (tidak ada koin wallet yang
 * dibelanjakan), keluar (saldo berkurang, termasuk biaya), atau ke alamat sendiri.
 */
export function classify(entry: HistoryEntryDto): { direction: Direction; net: bigint } {
  const net = toQuanta(entry.received) - toQuanta(entry.spent);
  if (entry.coinbase) {
    return { direction: 'mined', net };
  }
  if (toQuanta(entry.spent) === 0n) {
    return { direction: 'incoming', net };
  }
  return { direction: net < 0n ? 'outgoing' : 'self', net };
}

/** Jumlah konfirmasi di ujung `tipHeight`; 0 untuk transaksi tertunda. */
export function confirmations(height: number | null, tipHeight: number): number {
  if (height === null || height > tipHeight) {
    return 0;
  }
  return tipHeight - height + 1;
}

/** Entri yang cocok dengan saringan jenis dan pencarian txid (awalan atau bagian, huruf kecil). */
export function filterEntries(
  entries: HistoryEntryDto[],
  direction: Direction | 'all',
  search: string,
): HistoryEntryDto[] {
  const query = search.trim().toLowerCase();
  return entries.filter(
    (entry) =>
      (direction === 'all' || classify(entry).direction === direction) &&
      (query === '' || entry.txid.includes(query)),
  );
}
