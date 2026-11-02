'use client';

import React, { Suspense, useCallback } from 'react';
import Link from 'next/link';
import { useSearchParams } from 'next/navigation';
import { ArrowDown, ArrowRight, Clock } from 'lucide-react';
import { useApp } from '@/app/providers';
import { usePoll } from '@/hooks/use-poll';
import { fetchTx } from '@/lib/api';
import {
  computeFeeRateQuantaPerByte,
  formatDigitsWithThousands,
  formatInt,
} from '@/lib/amount';
import { isValidHash, routes, truncateMiddle } from '@/lib/route';
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
  HsisAmount,
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
                  <HsisAmount quanta={tx.fee} />
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

      {/* Visual Inputs -> Outputs Flow */}
      <Card>
        <CardHeader>
          <CardTitle>{t('txFlowTitle')}</CardTitle>
        </CardHeader>
        <CardContent className="p-5">
          <div className="grid grid-cols-1 lg:grid-cols-11 gap-4 items-start">
            {/* Left Column: Inputs */}
            <div className="lg:col-span-5 space-y-3">
              <div className="text-xs font-semibold text-slate-600 dark:text-slate-300 pb-1 border-b border-slate-200 dark:border-slate-800">
                {t('txInputsHeader', { count: formatInt(tx.inputs.length, lang) })}
              </div>

              {tx.is_coinbase || tx.inputs.length === 0 ? (
                <div className="rounded-lg border border-dashed border-slate-200 dark:border-slate-800 p-4 text-xs font-medium text-teal-700 dark:text-teal-400">
                  {t('txCoinbaseNoInputs')}
                </div>
              ) : (
                tx.inputs.map((inp, i) => (
                  <div
                    key={`${inp.txid}-${inp.index}-${i}`}
                    className="rounded-lg border border-slate-200 dark:border-slate-800 p-3.5 space-y-2 text-xs"
                  >
                    <div className="flex items-center justify-between gap-2">
                      <HashValue
                        value={inp.address}
                        href={routes.address(inp.address)}
                        startChars={12}
                        endChars={8}
                        label="address"
                      />
                      <span className="font-semibold text-slate-900 dark:text-slate-100">
                        <HsisAmount quanta={inp.amount} />
                      </span>
                    </div>
                    <div className="flex flex-wrap items-center gap-1.5 text-[11px] text-slate-500 dark:text-slate-400">
                      <span>{t('txInputSource', { index: inp.index })}:</span>
                      <HashValue
                        value={inp.txid}
                        href={routes.tx(inp.txid)}
                        startChars={10}
                        endChars={8}
                        label="source txid"
                      />
                    </div>
                    <div className="font-mono text-[10px] text-slate-400">
                      {t('txPayloadKindLabel', {
                        kind: inp.kind,
                        payload: truncateMiddle(inp.payload, 8, 6),
                      })}
                    </div>
                  </div>
                ))
              )}
            </div>

            {/* Center Arrow */}
            <div className="lg:col-span-1 flex items-center justify-center py-2 lg:py-8 text-slate-400">
              <ArrowRight className="hidden lg:block h-5 w-5" aria-hidden="true" />
              <ArrowDown className="lg:hidden h-5 w-5" aria-hidden="true" />
            </div>

            {/* Right Column: Outputs */}
            <div className="lg:col-span-5 space-y-3">
              <div className="text-xs font-semibold text-slate-600 dark:text-slate-300 pb-1 border-b border-slate-200 dark:border-slate-800">
                {t('txOutputsHeader', {
                  count: formatInt(tx.outputs.length, lang),
                })}
              </div>

              {tx.outputs.map((out) => (
                <div
                  key={out.index}
                  className="rounded-lg border border-slate-200 dark:border-slate-800 p-3.5 space-y-2 text-xs"
                >
                  <div className="flex items-center justify-between gap-2">
                    <span className="font-mono text-[11px] text-slate-500">
                      {t('txOutputIndex', { index: out.index })}
                    </span>
                    <span className="font-semibold text-emerald-700 dark:text-emerald-400">
                      <HsisAmount quanta={out.amount} />
                    </span>
                  </div>

                  <div>
                    <HashValue
                      value={out.address}
                      href={routes.address(out.address)}
                      startChars={14}
                      endChars={10}
                      label="address"
                    />
                  </div>

                  <div className="pt-1 flex items-center justify-between gap-2 text-[11px]">
                    {out.spent_by ? (
                      <span className="text-amber-700 dark:text-amber-400 font-medium">
                        {t('txSpentByBadge', {
                          txid: truncateMiddle(out.spent_by.txid, 8, 6),
                          height: formatInt(out.spent_by.height, lang),
                        })}{' '}
                        ·{' '}
                        <Link
                          href={routes.tx(out.spent_by.txid)}
                          className="underline font-mono"
                        >
                          tx →
                        </Link>
                      </span>
                    ) : (
                      <span className="text-emerald-700 dark:text-emerald-400 font-medium">
                        {t('txUnspentBadge')}
                      </span>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </div>
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
