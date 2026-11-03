'use client';

import React, { useCallback } from 'react';
import { Clock, RefreshCw } from 'lucide-react';
import { useApp } from '@/app/providers';
import { usePoll } from '@/hooks/use-poll';
import { fetchMempool } from '@/lib/api';
import { formatInt } from '@/lib/amount';
import { Alert, Card, CardContent } from '@/components/ui/primitives';
import { ApiStateView, TableSkeleton } from '@/components/explorer/shared';

export default function MempoolPage() {
  const { lang, t } = useApp();
  const fetcher = useCallback(() => fetchMempool(), []);
  const { data: mempool, failure, isLoading, isRefreshing, reload } = usePoll(
    fetcher,
    []
  );

  if (failure && !mempool) {
    return <ApiStateView failure={failure} onRetry={reload} />;
  }

  if (isLoading && !mempool) {
    return <TableSkeleton rows={8} />;
  }

  if (!mempool) return null;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-slate-200 dark:border-slate-800 pb-4">
        <div className="space-y-1">
          <h1 className="text-xl sm:text-2xl font-bold text-slate-900 dark:text-slate-100">
            {t('mempoolPageTitle')}
          </h1>
          <p className="text-xs text-slate-500 dark:text-slate-400">
            {t('mempoolPageSubtitle')}
          </p>
        </div>

        <div className="inline-flex items-center gap-2 font-mono text-xs text-teal-700 dark:text-teal-400">
          <RefreshCw
            className={`h-3.5 w-3.5 ${isRefreshing ? 'animate-spin' : ''}`}
          />
          <span>{t('liveUpdating')}</span>
        </div>
      </div>

      {/* Count & Notice */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 items-stretch">
        <Card>
          <CardContent className="p-5 space-y-1">
            <div className="text-xs text-slate-500 dark:text-slate-400">
              {t('mempoolCountLabel')}
            </div>
            <div className="text-3xl font-bold font-mono tabular-nums text-slate-900 dark:text-slate-100">
              {formatInt(mempool.count, lang)}
            </div>
          </CardContent>
        </Card>

        <Alert variant="info" className="lg:col-span-2 flex items-center">
          <div className="flex items-start gap-3">
            <Clock className="h-4 w-4 shrink-0 mt-0.5" />
            <p className="text-xs leading-relaxed">
              {mempool.count === 0 ? t('mempoolEmpty') : t('mempoolNotice')}
            </p>
          </div>
        </Alert>
      </div>
    </div>
  );
}
