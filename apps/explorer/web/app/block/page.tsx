'use client';

import React, { Suspense, useCallback } from 'react';
import Link from 'next/link';
import { useSearchParams } from 'next/navigation';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import { useApp } from '@/app/providers';
import { usePoll } from '@/hooks/use-poll';
import { fetchBlock } from '@/lib/api';
import { addQuanta, formatInt } from '@/lib/amount';
import { isValidBlockIdOrHeight, routes } from '@/lib/route';
import { formatRelativeTime, formatUtcTimestamp } from '@/lib/time';
import {
  Button,
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
  ConfirmationsBadge,
  HashValue,
  ZvrgAmount,
  TableSkeleton,
} from '@/components/explorer/shared';

function BlockDetailContent() {
  const { lang, t } = useApp();
  const searchParams = useSearchParams();
  const rawId = (searchParams.get('id') ?? '').trim().toLowerCase();

  const valid = isValidBlockIdOrHeight(rawId);

  const fetcher = useCallback(() => {
    if (!valid) {
      return Promise.resolve({
        ok: false as const,
        failure: { kind: 'http' as const, status: 400, error: 'bad_request' },
      });
    }
    return fetchBlock(rawId);
  }, [rawId, valid]);

  const { data: block, failure, isLoading, nowUnix, reload } = usePoll(fetcher, [
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

  if (failure && !block) {
    return <ApiStateView failure={failure} onRetry={reload} />;
  }

  if (isLoading && !block) {
    return <TableSkeleton rows={10} />;
  }

  if (!block) return null;

  const totalRewardQuanta = addQuanta(block.subsidy, block.fees) ?? 0n;
  const isGenesis = block.height === 0;

  return (
    <div className="space-y-6">
      {/* Header with height, confirmations, and Prev / Next buttons */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-slate-200 dark:border-slate-800 pb-4">
        <div className="space-y-1">
          <div className="flex flex-wrap items-center gap-3">
            <h1 className="text-xl sm:text-2xl font-bold font-mono tabular-nums text-slate-900 dark:text-slate-100">
              {t('blockDetailTitle', { height: formatInt(block.height, lang) })}
            </h1>
            {isGenesis && (
              <span className="text-xs font-medium text-teal-700 dark:text-teal-400">
                · {t('blockGenesisBadge')}
              </span>
            )}
            <ConfirmationsBadge confirmations={block.confirmations} />
          </div>
          <div className="flex items-center gap-2 text-xs text-slate-500 dark:text-slate-400">
            <HashValue
              value={block.block_id}
              truncate={false}
              label="block_id"
            />
          </div>
        </div>

        <div className="flex items-center gap-2 shrink-0">
          {block.height > 0 ? (
            <Button variant="outline" size="sm" asChild>
              <Link href={routes.block(block.height - 1)}>
                <ChevronLeft className="h-3.5 w-3.5" />
                <span>{t('blockPrevButton')}</span>
              </Link>
            </Button>
          ) : (
            <Button variant="outline" size="sm" disabled>
              <ChevronLeft className="h-3.5 w-3.5" />
              <span>{t('blockPrevButton')}</span>
            </Button>
          )}

          {block.next_block_id ? (
            <Button variant="outline" size="sm" asChild>
              <Link href={routes.block(block.next_block_id)}>
                <span>{t('blockNextButton')}</span>
                <ChevronRight className="h-3.5 w-3.5" />
              </Link>
            </Button>
          ) : (
            <Button variant="outline" size="sm" disabled>
              <span>{t('blockNextButton')}</span>
              <ChevronRight className="h-3.5 w-3.5" />
            </Button>
          )}
        </div>
      </div>

      {/* Block Metadata Grid */}
      <Card>
        <CardContent className="p-0">
          <dl className="divide-y divide-slate-100 dark:divide-slate-800/70 text-xs">
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-2 px-5 py-3">
              <dt className="font-medium text-slate-500 dark:text-slate-400">
                {t('blockFieldBlockId')}
              </dt>
              <dd className="sm:col-span-2">
                <HashValue value={block.block_id} truncate={false} label="block_id" />
              </dd>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-3 gap-2 px-5 py-3">
              <dt className="font-medium text-slate-500 dark:text-slate-400">
                {t('blockFieldPrevBlock')}
              </dt>
              <dd className="sm:col-span-2">
                {isGenesis ? (
                  <span className="font-mono text-slate-400">
                    {t('blockGenesisNoPrev')}
                  </span>
                ) : (
                  <HashValue
                    value={block.prev_block_id}
                    href={routes.block(block.prev_block_id)}
                    truncate={false}
                    label="prev_block_id"
                  />
                )}
              </dd>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-3 gap-2 px-5 py-3">
              <dt className="font-medium text-slate-500 dark:text-slate-400">
                {t('blockFieldNextBlock')}
              </dt>
              <dd className="sm:col-span-2">
                {block.next_block_id ? (
                  <HashValue
                    value={block.next_block_id}
                    href={routes.block(block.next_block_id)}
                    truncate={false}
                    label="next_block_id"
                  />
                ) : (
                  <span className="text-slate-400">{t('blockAtTipNoNext')}</span>
                )}
              </dd>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-3 gap-2 px-5 py-3">
              <dt className="font-medium text-slate-500 dark:text-slate-400">
                {t('blockFieldMerkleRoot')}
              </dt>
              <dd className="sm:col-span-2">
                <HashValue
                  value={block.merkle_root}
                  truncate={false}
                  label="merkle_root"
                />
              </dd>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-3 gap-2 px-5 py-3">
              <dt className="font-medium text-slate-500 dark:text-slate-400">
                {t('blockFieldTimestamp')}
              </dt>
              <dd className="sm:col-span-2 font-mono tabular-nums">
                <span>{formatUtcTimestamp(block.timestamp)}</span>
                <span className="ml-2 font-sans text-slate-500 dark:text-slate-400">
                  ({formatRelativeTime(block.timestamp, nowUnix, lang)})
                </span>
              </dd>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-3 gap-2 px-5 py-3">
              <dt className="font-medium text-slate-500 dark:text-slate-400">
                {t('blockFieldTarget')} / {t('blockFieldNonce')}
              </dt>
              <dd className="sm:col-span-2 font-mono tabular-nums">
                <span>0x{block.target}</span>
                <span className="mx-2 text-slate-300 dark:text-slate-700">·</span>
                <span>nonce: {block.nonce}</span>
              </dd>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-3 gap-2 px-5 py-3">
              <dt className="font-medium text-slate-500 dark:text-slate-400">
                {t('blockFieldSize')} / {t('blockFieldTxCount')}
              </dt>
              <dd className="sm:col-span-2 font-mono tabular-nums">
                <span>{formatInt(block.size, lang)} bytes</span>
                <span className="mx-2 text-slate-300 dark:text-slate-700">·</span>
                <span>{formatInt(block.tx_count, lang)} tx</span>
              </dd>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-3 gap-2 px-5 py-3">
              <dt className="font-medium text-slate-500 dark:text-slate-400">
                {t('blockFieldSubsidy')}
              </dt>
              <dd className="sm:col-span-2">
                <ZvrgAmount quanta={block.subsidy} />
              </dd>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-3 gap-2 px-5 py-3">
              <dt className="font-medium text-slate-500 dark:text-slate-400">
                {t('blockFieldFees')}
              </dt>
              <dd className="sm:col-span-2">
                <ZvrgAmount quanta={block.fees} />
              </dd>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-3 gap-2 px-5 py-3 bg-slate-50/60 dark:bg-slate-900/40">
              <dt className="font-semibold text-slate-700 dark:text-slate-200">
                {t('blockFieldTotalReward')}
              </dt>
              <dd className="sm:col-span-2 font-semibold text-teal-700 dark:text-teal-400">
                <ZvrgAmount quanta={totalRewardQuanta} />
              </dd>
            </div>
          </dl>
        </CardContent>
      </Card>

      {/* Transactions in Block */}
      <Card>
        <CardHeader>
          <CardTitle>
            {t('blockTransactionsTitle')} ({formatInt(block.txids.length, lang)})
          </CardTitle>
        </CardHeader>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-20">#</TableHead>
              <TableHead>{t('colTxid')}</TableHead>
              <TableHead className="text-right">Role</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {block.txids.map((txid, idx) => (
              <TableRow key={txid}>
                <TableCell className="font-mono text-slate-500 tabular-nums">
                  {idx}
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
                  {idx === 0 ? (
                    <span className="font-medium text-teal-700 dark:text-teal-400">
                      {t('blockCoinbaseLabel')}
                    </span>
                  ) : (
                    <span className="text-slate-500 dark:text-slate-400">
                      UTXO Transfer
                    </span>
                  )}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </Card>
    </div>
  );
}

export default function BlockDetailPage() {
  return (
    <Suspense fallback={<TableSkeleton rows={10} />}>
      <BlockDetailContent />
    </Suspense>
  );
}
