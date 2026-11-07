/**
 * Tabel riwayat transaksi, dipakai layar Ringkasan (5 terbaru) dan Riwayat.
 */

import { ArrowDownLeft, ArrowUpRight, Pickaxe, RefreshCw } from 'lucide-react';
import { type Direction, type HistoryEntryDto, classify, confirmations } from '../lib/history';
import type { Dict } from '../lib/i18n';
import { type Lang, toQuanta } from '../lib/quanta';
import { CopyButton, Zvrg } from './ui';

const STYLE: Record<Direction, string> = {
  mined: 'text-amber-600 dark:text-amber-400',
  incoming: 'text-emerald-600 dark:text-emerald-400',
  outgoing: 'text-rose-600 dark:text-rose-400',
  self: 'text-slate-600 dark:text-slate-300',
};

function DirectionLabel({ direction, t }: { direction: Direction; t: Dict }) {
  const Icon = { mined: Pickaxe, incoming: ArrowDownLeft, outgoing: ArrowUpRight, self: RefreshCw }[direction];
  const label = { mined: t.filterMined, incoming: t.filterIncoming, outgoing: t.filterOutgoing, self: t.filterSelf }[
    direction
  ];
  return (
    <span className={`inline-flex items-center gap-1.5 font-semibold ${STYLE[direction]}`}>
      <Icon className="w-3.5 h-3.5" />
      {label}
    </span>
  );
}

export function HistoryTable({
  entries,
  tipHeight,
  t,
  lang,
  exact,
  emptyText,
}: {
  entries: HistoryEntryDto[];
  tipHeight: number;
  t: Dict;
  lang: Lang;
  exact: boolean;
  emptyText: string;
}) {
  const locale = lang === 'id' ? 'id-ID' : 'en-US';
  return (
    <div className="overflow-x-auto">
      <table className="w-full text-left border-collapse text-xs">
        <thead>
          <tr className="border-b border-slate-200 dark:border-slate-800 text-[11px] font-semibold text-slate-500 dark:text-slate-400">
            <th className="py-2.5 px-4">{t.colType}</th>
            <th className="py-2.5 px-4 text-right">{t.colAmount}</th>
            <th className="py-2.5 px-4 text-right">{t.colFee}</th>
            <th className="py-2.5 px-4">{t.colStatus}</th>
            <th className="py-2.5 px-4">{t.colTime}</th>
            <th className="py-2.5 px-4">{t.colTxid}</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-slate-200 dark:divide-slate-800">
          {entries.length === 0 ? (
            <tr>
              <td colSpan={6} className="py-10 px-4 text-center text-slate-500 dark:text-slate-400">
                {emptyText}
              </td>
            </tr>
          ) : (
            entries.map((entry) => {
              const { direction, net } = classify(entry);
              const conf = confirmations(entry.height, tipHeight);
              return (
                <tr key={entry.txid} className="hover:bg-slate-50 dark:hover:bg-slate-800/40">
                  <td className="py-3 px-4 whitespace-nowrap">
                    <DirectionLabel direction={direction} t={t} />
                  </td>
                  <td className={`py-3 px-4 text-right whitespace-nowrap font-semibold ${STYLE[direction]}`}>
                    <Zvrg quanta={net} lang={lang} exact={exact} signed />
                  </td>
                  <td className="py-3 px-4 text-right whitespace-nowrap text-slate-500 dark:text-slate-400">
                    {entry.fee === null ? '—' : <Zvrg quanta={toQuanta(entry.fee)} lang={lang} exact />}
                  </td>
                  <td className="py-3 px-4 whitespace-nowrap">
                    {entry.height === null ? (
                      <span className="font-semibold text-amber-600 dark:text-amber-400">{t.statusPending}</span>
                    ) : (
                      <span className="text-slate-600 dark:text-slate-300 tabular-nums">
                        {conf.toLocaleString(locale)} {t.confirmationsLabel} · {t.blockLabel}{' '}
                        {entry.height.toLocaleString(locale)}
                      </span>
                    )}
                  </td>
                  <td className="py-3 px-4 whitespace-nowrap tabular-nums text-slate-500 dark:text-slate-400">
                    {entry.timestamp === null ? '—' : new Date(entry.timestamp * 1000).toLocaleString(locale)}
                  </td>
                  <td className="py-3 px-4 whitespace-nowrap font-mono">
                    <div className="flex items-center gap-2">
                      <span title={entry.txid} className="text-slate-700 dark:text-slate-300">
                        {entry.txid.slice(0, 10)}…{entry.txid.slice(-6)}
                      </span>
                      <CopyButton text={entry.txid} t={t} />
                    </div>
                  </td>
                </tr>
              );
            })
          )}
        </tbody>
      </table>
    </div>
  );
}
