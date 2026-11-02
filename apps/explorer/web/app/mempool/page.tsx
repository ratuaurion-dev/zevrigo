'use client';

import React, { useCallback } from 'react';
import Link from 'next/link';
import { Clock, RefreshCw } from 'lucide-react';
import { useApp } from '@/app/providers';
import { usePoll } from '@/hooks/use-poll';
import { fetchMempool } from '@/lib/api';
import { formatInt } from '@/lib/amount';
import { routes } from '@/lib/route';
import {
  Alert,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/primitives';
import {
  ApiStateView,
  HashValue,
  TableSkeleton,
} from '@/components/explorer/shared';

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
            <p className="text-xs leading-relaxed">{t('mempoolNotice')}</p>
          </div>
        </Alert>
      </div>

      {/* Mempool txid Table */}
      <Card>
        <CardHeader>
          <CardTitle>
            {t('mempoolTxListTitle')} ({formatInt(mempool.txids.length, lang)})
          </CardTitle>
        </CardHeader>

        {mempool.txids.length === 0 ? (
          <div className="p-8 text-center text-xs text-slate-500">
            {t('mempoolEmpty')}
          </div>
        ) : (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead className="w-16">#</TableHead>
                <TableHead>{t('colTxid')}</TableHead>
                <TableHead className="text-right">{t('colStatus')}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {mempool.txids.map((txid, idx) => (
                <TableRow key={txid}>
                  <TableCell className="font-mono text-slate-400 tabular-nums">
                    {idx + 1}
                  </TableCell>
                  <TableCell>
                    <HashValue
                      value={txid}
                      href={routes.tx(txid)}
                      truncate={false}
                      label="txid"
                    />
                  </TableCell>
                  <TableCell className="text-right">
                    <Link
                      href={routes.tx(txid)}
                      className="inline-flex items-center gap-1.5 text-amber-700 dark:text-amber-400 font-medium hover:underline"
                    >
                      <Clock className="h-3.5 w-3.5" />
                      <span>{t('txStatusPendingTitle')}</span>
                    </Link>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
      </Card>
    </div>
  );
}
