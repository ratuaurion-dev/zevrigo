'use client';

import React, { Suspense, useCallback } from 'react';
import Link from 'next/link';
import { useSearchParams } from 'next/navigation';
import { Clock } from 'lucide-react';
import { useApp } from '@/app/providers';
import { usePoll } from '@/hooks/use-poll';
import { fetchTx } from '@/lib/api';
import {
  computeFeeRateQuantaPerByte,
  formatDigitsWithThousands,
  formatInt,
} from '@/lib/amount';
import { isValidHash, routes } from '@/lib/route';
import {
  Alert,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from '@/components/ui/primitives';
import {
  ApiStateView,
  ConfirmationsBadge,
  HashValue,
  ZvrgAmount,
  TableSkeleton,
} from '@/components/explorer/shared';

function TxDetailContent() {
  const { lang, t } = useApp();
  const searchParams = useSearchParams();
  const rawId = (searchParams.get('id') ?? '').trim().toLowerCase();

  const valid = isValidHash(rawId);

  const fetcher = useCallback(() => {
    if (!valid) {
      return Promise.resolve({
        ok: false as const,
        failure: { kind: 'http' as const, status: 400, error: 'bad_request' },
      });
    }
    return fetchTx(rawId);
  }, [rawId, valid]);

  const { data: tx, failure, isLoading, reload } = usePoll(fetcher, [
    rawId,
    valid,
  ]);

  if (!valid) {
    return (
      <ApiStateView
        failure={{ kind: 'http', status: 400, error: 'bad_request' }}
      />
    );
  }

  // Friendly pending state when 404 has `in_mempool: true`
  if (
    failure &&
    failure.kind === 'http' &&
    failure.status === 404 &&
    failure.inMempool === true
  ) {
    return (
      <div className="space-y-5">
        <div className="border-b border-slate-200 dark:border-slate-800 pb-4 space-y-1">
          <h1 className="text-xl font-bold text-slate-900 dark:text-slate-100">
            {t('txDetailTitle')}
          </h1>
          <HashValue value={rawId} truncate={false} label="txid" />
        </div>

        <Alert variant="warning" className="p-5">
          <div className="flex items-start gap-3">
            <Clock className="h-5 w-5 shrink-0 mt-0.5 text-amber-600 dark:text-amber-400" />
            <div className="space-y-1">
              <div className="text-sm font-semibold">
                {t('txStatusPendingTitle')}
              </div>
              <p className="text-xs leading-relaxed">
                {t('txStatusPendingBody')}
              </p>
              <div className="pt-2">
                <Link
                  href={routes.mempool()}
                  className="text-xs font-semibold underline hover:opacity-80"
                >
                  {t('navMempool')} →
                </Link>
              </div>
            </div>
          </div>
        </Alert>
      </div>
    );
  }

  if (failure && !tx) {
    return <ApiStateView failure={failure} onRetry={reload} />;
  }

  if (isLoading && !tx) {
    return <TableSkeleton rows={10} />;
  }

  if (!tx) return null;

  const feeRate = tx.is_coinbase
    ? null
    : computeFeeRateQuantaPerByte(tx.fee, tx.size);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-slate-200 dark:border-slate-800 pb-4">
        <div className="space-y-1">
          <div className="flex flex-wrap items-center gap-3">
            <h1 className="text-xl font-bold text-slate-900 dark:text-slate-100">
              {t('txDetailTitle')}
            </h1>
            {tx.is_coinbase && (
              <span className="text-xs font-medium text-teal-700 dark:text-teal-400">
                · {t('blockCoinbaseLabel')}
              </span>
            )}
            <ConfirmationsBadge confirmations={tx.confirmations} />
          </div>
          <HashValue value={tx.txid} truncate={false} label="txid" />
        </div>
      </div>

      {/* Metadata Card */}
      <Card>
        <CardContent className="p-0">
          <dl className="divide-y divide-slate-100 dark:divide-slate-800/70 text-xs">
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-2 px-5 py-3">
              <dt className="font-medium text-slate-500 dark:text-slate-400">
                {t('txFieldBlock')}
              </dt>
              <dd className="sm:col-span-2 flex flex-wrap items-center gap-2">
                <Link
                  href={routes.block(tx.height)}
                  className="font-mono font-semibold text-teal-700 dark:text-teal-400 hover:underline"
                >
                  #{formatInt(tx.height, lang)}
                </Link>
                <span aria-hidden="true">·</span>
                <HashValue
                  value={tx.block_id}
                  href={routes.block(tx.block_id)}
                  startChars={12}
                  endChars={10}
                  label="block_id"
                />
              </dd>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-3 gap-2 px-5 py-3">
              <dt className="font-medium text-slate-500 dark:text-slate-400">
                {t('txFieldPosition')} / {t('txFieldSize')}
              </dt>
              <dd className="sm:col-span-2 font-mono tabular-nums">
                <span>#{formatInt(tx.position, lang)}</span>
                <span className="mx-2 text-slate-300 dark:text-slate-700">·</span>
                <span>{formatInt(tx.size, lang)} bytes</span>
              </dd>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-3 gap-2 px-5 py-3">
              <dt className="font-medium text-slate-500 dark:text-slate-400">
                {t('txFieldFee')}
              </dt>
              <dd className="sm:col-span-2">
                {tx.is_coinbase ? (
                  <span className="text-slate-500 dark:text-slate-400">
                    {t('txFieldFeeNone')}
                  </span>
                ) : (
                  <ZvrgAmount quanta={tx.fee} />
                )}
              </dd>
            </div>

            {!tx.is_coinbase && feeRate !== null && (
              <div className="grid grid-cols-1 sm:grid-cols-3 gap-2 px-5 py-3">
                <dt className="font-medium text-slate-500 dark:text-slate-400">
                  {t('txFieldFeeRate')}
                </dt>
                <dd className="sm:col-span-2 font-mono tabular-nums">
                  {formatDigitsWithThousands(feeRate.toString(10), lang)} quanta/byte
                </dd>
              </div>
            )}
          </dl>
        </CardContent>
      </Card>

      {/* Ringkasan input dan output: tanpa alamat, jumlah per output, maupun jejak koin (DEC-28) */}
      <Card>
        <CardHeader>
          <CardTitle>{t('txFlowTitle')}</CardTitle>
        </CardHeader>
        <CardContent className="p-5 space-y-4">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div className="rounded-lg border border-slate-200 dark:border-slate-800 p-4 space-y-1">
              <div className="text-xs text-slate-500 dark:text-slate-400">
                {t('txInputCountLabel')}
              </div>
              <div className="text-2xl font-bold font-mono tabular-nums text-slate-900 dark:text-slate-100">
                {formatInt(tx.input_count, lang)}
              </div>
              {tx.is_coinbase && (
                <div className="text-[11px] font-medium text-teal-700 dark:text-teal-400">
                  {t('txCoinbaseNoInputs')}
                </div>
              )}
            </div>
            <div className="rounded-lg border border-slate-200 dark:border-slate-800 p-4 space-y-1">
              <div className="text-xs text-slate-500 dark:text-slate-400">
                {t('txOutputCountLabel')}
              </div>
              <div className="text-2xl font-bold font-mono tabular-nums text-slate-900 dark:text-slate-100">
                {formatInt(tx.output_count, lang)}
              </div>
            </div>
          </div>
          <p className="text-[11px] leading-relaxed text-slate-500 dark:text-slate-400">
            {t('txPrivacyNote')}
          </p>
        </CardContent>
      </Card>
    </div>
  );
}

export default function TxDetailPage() {
  return (
    <Suspense fallback={<TableSkeleton rows={10} />}>
      <TxDetailContent />
    </Suspense>
  );
}
