/**
 * Ringkasan: saldo (bisa dipakai, belum matang, tertunda), alamat penerimaan terakhir, dan 5 transaksi terbaru.
 */

import { ArrowDownLeft, ArrowUpRight, Binary, Clock, Lock, Shield } from 'lucide-react';
import type { ReactNode } from 'react';
import { useState } from 'react';
import { TEXT } from '../lib/i18n';
import { toQuanta } from '../lib/quanta';
import { useWallet } from '../store';
import { HistoryTable } from './HistoryTable';
import { Offline } from './Offline';
import { Card, CopyButton, PageHeader, Zvrg, primaryButton, secondaryButton } from './ui';

function BalanceCard({ title, hint, icon, children }: { title: string; hint: string; icon: ReactNode; children: ReactNode }) {
  return (
    <Card className="flex flex-col justify-between gap-4">
      <div>
        <div className="flex items-center justify-between text-xs font-semibold text-slate-500 dark:text-slate-400">
          <span>{title}</span>
          {icon}
        </div>
        <div className="mt-3 text-xl lg:text-2xl font-bold text-slate-900 dark:text-slate-50 break-all">{children}</div>
      </div>
      <p className="text-xs text-slate-500 dark:text-slate-400">{hint}</p>
    </Card>
  );
}

export function Dashboard() {
  const { language, balance, history, address, connection, setTab } = useWallet();
  const t = TEXT[language];
  const [exact, setExact] = useState(false);
  const z = (text: string) => <Zvrg quanta={toQuanta(text)} lang={language} exact={exact} />;

  return (
    <div className="space-y-6">
      <PageHeader
        title={t.dashboardTitle}
        action={
          <button type="button" onClick={() => setExact(!exact)} aria-pressed={exact} className={secondaryButton}>
            <Binary className="w-3.5 h-3.5" />
            {t.showExact}
          </button>
        }
      />

      {connection === 'offline' && <Offline />}

      {balance && (
        <>
          <Card className="flex flex-col md:flex-row md:items-end justify-between gap-3">
            <div>
              <div className="text-xs font-semibold text-slate-500 dark:text-slate-400">{t.balanceTotal}</div>
              <div data-testid="balance-total" className="mt-1 text-3xl font-bold text-slate-900 dark:text-slate-50 break-all">
                {z(balance.total)}
              </div>
            </div>
            <div className="text-xs text-slate-500 dark:text-slate-400 md:text-right space-y-0.5">
              <div>
                {t.balanceAfterPending}: {z(balance.afterPending)}
              </div>
              <div>
                {t.utxoCount}: <span className="font-mono tabular-nums">{balance.utxos}</span>
              </div>
            </div>
          </Card>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <BalanceCard
              title={t.balanceSpendable}
              hint={t.balanceSpendableHint}
              icon={<Shield className="w-4 h-4 text-emerald-500" />}
            >
              {z(balance.spendable)}
            </BalanceCard>
            <BalanceCard
              title={t.balanceImmature}
              hint={t.balanceImmatureHint}
              icon={<Lock className="w-4 h-4 text-slate-400" />}
            >
              {z(balance.immature)}
            </BalanceCard>
            <BalanceCard
              title={t.balancePending}
              hint={t.balancePendingHint}
              icon={<Clock className="w-4 h-4 text-amber-500" />}
            >
              <div className="text-base space-y-1">
                <div className="text-emerald-600 dark:text-emerald-400">
                  + <Zvrg quanta={toQuanta(balance.pendingReceived)} lang={language} exact={exact} />
                </div>
                <div className="text-rose-600 dark:text-rose-400">
                  − <Zvrg quanta={toQuanta(balance.pendingSpent)} lang={language} exact={exact} />
                </div>
              </div>
            </BalanceCard>
          </div>
        </>
      )}

      {!balance && connection !== 'offline' && (
        <Card className="text-sm text-slate-500 dark:text-slate-400">{t.loading}</Card>
      )}

      <Card className="flex flex-col lg:flex-row lg:items-center justify-between gap-4">
        <div className="space-y-1 min-w-0">
          <div className="text-xs font-medium text-slate-500 dark:text-slate-400">{t.receiveAddress}</div>
          <div data-testid="dashboard-address" className="font-mono text-sm font-semibold text-slate-900 dark:text-slate-100 break-all">
            {address}
          </div>
        </div>
        <div className="flex flex-wrap items-center gap-2.5 shrink-0">
          <CopyButton text={address} t={t} />
          <button type="button" onClick={() => setTab('receive')} className={secondaryButton}>
            <ArrowDownLeft className="w-3.5 h-3.5 text-emerald-500" />
            {t.navReceive}
          </button>
          <button type="button" onClick={() => setTab('send')} className={primaryButton}>
            <ArrowUpRight className="w-4 h-4" />
            {t.navSend}
          </button>
        </div>
      </Card>

      {history && (
        <Card flush>
          <div className="px-5 py-4 border-b border-slate-200 dark:border-slate-800 flex items-center justify-between">
            <h2 className="text-sm font-semibold text-slate-900 dark:text-slate-100">{t.recentTitle}</h2>
            <button
              type="button"
              onClick={() => setTab('history')}
              className="text-xs font-semibold text-emerald-600 dark:text-emerald-400 hover:underline"
            >
              {t.viewAll}
            </button>
          </div>
          <HistoryTable
            entries={history.entries.slice(0, 5)}
            tipHeight={history.tipHeight}
            t={t}
            lang={language}
            exact={exact}
            emptyText={t.historyEmpty}
          />
        </Card>
      )}
    </div>
  );
}
