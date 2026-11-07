/**
 * Riwayat: semua transaksi wallet dari backend (tertunda dulu, lalu dari blok tertinggi), saringan jenis, cari txid.
 */

import { RefreshCw, Search } from 'lucide-react';
import { useMemo, useState } from 'react';
import { type Direction, filterEntries } from '../lib/history';
import { TEXT } from '../lib/i18n';
import { useWallet } from '../store';
import { HistoryTable } from './HistoryTable';
import { Offline } from './Offline';
import { Card, PageHeader, inputClass, secondaryButton } from './ui';

export function History() {
  const { language, history, connection, refresh, refreshing } = useWallet();
  const t = TEXT[language];
  const [direction, setDirection] = useState<Direction | 'all'>('all');
  const [search, setSearch] = useState('');
  const entries = useMemo(
    () => (history ? filterEntries(history.entries, direction, search) : []),
    [history, direction, search],
  );
  const filters: { key: Direction | 'all'; label: string }[] = [
    { key: 'all', label: t.filterAll },
    { key: 'incoming', label: t.filterIncoming },
    { key: 'outgoing', label: t.filterOutgoing },
    { key: 'mined', label: t.filterMined },
    { key: 'self', label: t.filterSelf },
  ];

  return (
    <div className="space-y-6">
      <PageHeader
        title={t.historyTitle}
        subtitle={t.historySubtitle}
        action={
          <button type="button" onClick={() => refresh()} disabled={refreshing} className={secondaryButton}>
            <RefreshCw className={`w-3.5 h-3.5 ${refreshing ? 'animate-spin' : ''}`} />
            {t.retry}
          </button>
        }
      />
      {connection === 'offline' && <Offline />}
      <div className="flex flex-col md:flex-row md:items-center gap-3">
        <div role="tablist" className="inline-flex flex-wrap p-1 rounded-lg bg-slate-200/80 dark:bg-slate-900 border border-slate-300/80 dark:border-slate-800">
          {filters.map((f) => (
            <button
              key={f.key}
              type="button"
              role="tab"
              aria-selected={direction === f.key}
              onClick={() => setDirection(f.key)}
              className={`px-3 py-1.5 text-xs font-semibold rounded-md whitespace-nowrap ${
                direction === f.key
                  ? 'bg-white dark:bg-slate-800 text-slate-900 dark:text-emerald-400'
                  : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200'
              }`}
            >
              {f.label}
            </button>
          ))}
        </div>
        <div className="relative flex-1">
          <Search className="w-4 h-4 text-slate-400 absolute left-3.5 top-1/2 -translate-y-1/2" />
          <input
            type="search"
            aria-label={t.searchTxid}
            placeholder={t.searchTxid}
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className={`${inputClass} pl-10 font-mono`}
          />
        </div>
      </div>
      <Card flush>
        {history ? (
          <HistoryTable
            entries={entries}
            tipHeight={history.tipHeight}
            t={t}
            lang={language}
            exact
            emptyText={history.entries.length === 0 ? t.historyEmpty : t.historyNone}
          />
        ) : (
          <div className="p-5 text-sm text-slate-500 dark:text-slate-400">{t.loading}</div>
        )}
      </Card>
    </div>
  );
}
