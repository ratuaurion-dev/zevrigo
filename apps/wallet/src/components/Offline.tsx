/**
 * Pesan saat node tidak bisa dihubungi atau berjalan di jaringan lain.
 */

import { CircleAlert } from 'lucide-react';
import { TEXT, walletErrorText } from '../lib/i18n';
import { useWallet } from '../store';
import { secondaryButton } from './ui';

export function Offline() {
  const { language, syncError, refresh, refreshing, rpcUrl } = useWallet();
  const t = TEXT[language];
  return (
    <div className="flex flex-col sm:flex-row sm:items-start justify-between gap-4 p-4 rounded-xl bg-rose-500/10 border border-rose-500/30">
      <div className="flex items-start gap-3 text-sm">
        <CircleAlert className="w-5 h-5 text-rose-500 shrink-0 mt-0.5" />
        <div className="space-y-1">
          <div className="font-semibold text-rose-700 dark:text-rose-300">{t.offlineTitle}</div>
          <div className="text-slate-700 dark:text-slate-300">
            {syncError ? walletErrorText(t, syncError.code) : ''} {t.offlineBody}
          </div>
          <div className="font-mono text-xs text-slate-500 dark:text-slate-400">
            {rpcUrl()} · {t.offlineCommand}
          </div>
          {syncError && <div className="font-mono text-[11px] text-slate-500 break-all">{syncError.message}</div>}
        </div>
      </div>
      <button type="button" onClick={() => refresh()} disabled={refreshing} className={secondaryButton}>
        {t.retry}
      </button>
    </div>
  );
}
