// Komponen tampilan bersama: jumlah koin, angka, hash, konfirmasi, galat, daftar isian, dan halaman.
import type { ReactNode } from 'react';
import { formatHsis, groupDigits, parseQuanta, separators } from '../amount';
import type { BlockRow, Failure } from '../api';
import type { Dict, Lang } from '../i18n';
import { fill } from '../i18n';
import type { Load } from '../load';
import { href } from '../route';
import { formatUtc } from '../time';
import type { Ui } from '../ui';

/** Jumlah konfirmasi minimum sebelum blok atau transaksi tidak lagi ditandai belum aman. */
export const SAFE_CONFIRMATIONS = 6;

export function Amount({ quanta, lang }: { quanta: string; lang: Lang }) {
  const value = parseQuanta(quanta);
  if (value === null) {
    return <span className="amount">—</span>;
  }
  return (
    <span className="amount" title={`${quanta} quanta`}>
      {formatHsis(value, lang)}
    </span>
  );
}

export function Count({ value, lang }: { value: number; lang: Lang }) {
  return <>{groupDigits(String(value), separators(lang).thousands)}</>;
}

/** Hash 64 karakter disingkat untuk tabel; nilai lengkap di `title`. */
export function shortHash(value: string): string {
  return value.length > 20 ? `${value.slice(0, 10)}…${value.slice(-8)}` : value;
}

export function HashText({ value }: { value: string }) {
  return <code className="hash">{value}</code>;
}

export function BlockLink({ id, short }: { id: string; short?: boolean }) {
  return (
    <a className="hash" href={href({ name: 'block', key: id })} title={id}>
      {short === true ? shortHash(id) : id}
    </a>
  );
}

export function TxLink({ txid, short }: { txid: string; short?: boolean }) {
  return (
    <a className="hash" href={href({ name: 'tx', txid })} title={txid}>
      {short === true ? shortHash(txid) : txid}
    </a>
  );
}

export function AddressLink({ address }: { address: string }) {
  return (
    <a className="hash" href={href({ name: 'address', address, page: '0' })}>
      {address}
    </a>
  );
}

export function HeightLink({ height, lang }: { height: number; lang: Lang }) {
  return (
    <a href={href({ name: 'block', key: String(height) })}>
      <Count value={height} lang={lang} />
    </a>
  );
}

export function Confirmations({ n, t }: { n: number; t: Dict }) {
  const low = n < SAFE_CONFIRMATIONS;
  return (
    <span className={low ? 'conf conf-low' : 'conf'}>
      {fill(t.confirmations, { n: String(n) })}
      {low ? ` · ${t.notFinal}` : ''}
    </span>
  );
}

/** Teks untuk kegagalan permintaan. */
export function failureText(failure: Failure, t: Dict): string {
  if (failure.kind !== 'http') {
    return t.loadFailed;
  }
  if (failure.status === 404) {
    return failure.inMempool ? t.inMempool : t.notFound;
  }
  if (failure.status === 503) {
    return t.notReady;
  }
  if (failure.status === 400) {
    return t.badRequest;
  }
  return t.loadFailed;
}

export function FailureBox({ failure, t }: { failure: Failure; t: Dict }) {
  const waiting = failure.kind === 'http' && failure.inMempool;
  return <p className={waiting ? 'notice' : 'notice notice-error'}>{failureText(failure, t)}</p>;
}

export function Loaded<T>({ load, t, children }: { load: Load<T>; t: Dict; children: (data: T) => ReactNode }) {
  if (load.status === 'loading') {
    return <p className="muted">{t.loading}</p>;
  }
  if (load.status === 'error') {
    return <FailureBox failure={load.failure} t={t} />;
  }
  return <>{children(load.data)}</>;
}

export type FieldRow = [string, ReactNode];

export function Fields({ rows }: { rows: FieldRow[] }) {
  return (
    <dl className="fields">
      {rows.map(([label, value]) => (
        <div className="field" key={label}>
          <dt>{label}</dt>
          <dd>{value}</dd>
        </div>
      ))}
    </dl>
  );
}

export function Pager({ prev, next, prevLabel, nextLabel }: { prev: string | null; next: string | null; prevLabel: string; nextLabel: string }) {
  if (prev === null && next === null) {
    return null;
  }
  return (
    <nav className="pager">
      {prev !== null && <a href={prev}>← {prevLabel}</a>}
      {next !== null && <a href={next}>{nextLabel} →</a>}
    </nav>
  );
}

export function BlocksTable({ rows, ui }: { rows: BlockRow[]; ui: Ui }) {
  const { t, lang } = ui;
  if (rows.length === 0) {
    return <p className="muted">{t.noBlocks}</p>;
  }
  return (
    <table className="table">
      <thead>
        <tr>
          <th>{t.height}</th>
          <th>{t.blockId}</th>
          <th>{t.time}</th>
          <th>{t.txCount}</th>
          <th>{t.size}</th>
          <th>{t.confirmationsHeader}</th>
        </tr>
      </thead>
      <tbody>
        {rows.map((row) => (
          <tr key={row.block_id}>
            <td>
              <HeightLink height={row.height} lang={lang} />
            </td>
            <td>
              <BlockLink id={row.block_id} short />
            </td>
            <td>{formatUtc(row.timestamp)}</td>
            <td>
              <Count value={row.tx_count} lang={lang} />
            </td>
            <td>
              <Count value={row.size} lang={lang} />
            </td>
            <td>
              <Confirmations n={row.confirmations} t={t} />
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
