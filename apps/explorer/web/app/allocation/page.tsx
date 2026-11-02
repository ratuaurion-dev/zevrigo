'use client';

import React, { Suspense, useCallback } from 'react';
import Link from 'next/link';
import { useSearchParams } from 'next/navigation';
import { ArrowRight, Eye } from 'lucide-react';
import { useApp } from '@/app/providers';
import { usePoll } from '@/hooks/use-poll';
import { fetchAllocation } from '@/lib/api';
import {
  computeBasisPoints,
  formatBasisPointsPercent,
  formatInt,
  parseQuanta,
  subQuanta,
} from '@/lib/amount';
import { isValidPage, routes, truncateMiddle } from '@/lib/route';
import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Progress,
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
  HsisAmount,
  TableSkeleton,
} from '@/components/explorer/shared';

function AllocationPageContent() {
  const { lang, t } = useApp();
  const searchParams = useSearchParams();
  const rawPage = searchParams.get('page') ?? '0';

  const validPage = isValidPage(rawPage);
  const pageNum = validPage ? Number(rawPage) : 0;

  const fetcher = useCallback(() => {
    if (!validPage) {
      return Promise.resolve({
        ok: false as const,
        failure: { kind: 'http' as const, status: 400, error: 'bad_request' },
      });
    }
    return fetchAllocation(pageNum);
  }, [pageNum, validPage]);

  const { data: alloc, failure, isLoading, reload } = usePoll(fetcher, [
    pageNum,
    validPage,
  ]);

  if (!validPage) {
    return (
      <ApiStateView
        failure={{ kind: 'http', status: 400, error: 'bad_request' }}
      />
    );
  }

  if (failure && !alloc) {
    return <ApiStateView failure={failure} onRetry={reload} />;
  }

  if (isLoading && !alloc) {
    return <TableSkeleton rows={10} />;
  }

  if (!alloc) return null;

  const movedQuanta = subQuanta(alloc.allocated, alloc.unspent) ?? 0n;
  const movedBps = computeBasisPoints(movedQuanta, alloc.allocated);
  const movedPct = formatBasisPointsPercent(movedQuanta, alloc.allocated, lang);
  const unspentPct = formatBasisPointsPercent(
    alloc.unspent,
    alloc.allocated,
    lang
  );

  return (
    <div className="space-y-6">
      {/* Transparency Hero Card */}
      <Card>
        <CardContent className="p-6 space-y-4">
          <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-4">
            <div className="space-y-1.5 max-w-3xl">
              <div className="inline-flex items-center gap-2 text-xs font-semibold text-teal-700 dark:text-teal-400">
                <Eye className="h-4 w-4 shrink-0" />
                <span>{t('allocationPageTitle')}</span>
              </div>
              <h1 className="text-xl sm:text-2xl font-bold text-slate-900 dark:text-slate-100">
                {t('allocationHeroTitle')}
              </h1>
              <p className="text-xs text-slate-600 dark:text-slate-300 leading-relaxed">
                {t('allocationHeroBody')}
              </p>
            </div>

            <div className="shrink-0">
              <Button variant="outline" asChild>
                <Link href={routes.block(alloc.genesis_block_id)}>
                  <span>{t('allocationGenesisBlockLink')}</span>
                  <ArrowRight className="h-3.5 w-3.5" />
                </Link>
              </Button>
            </div>
          </div>

          <div className="pt-3 border-t border-slate-100 dark:border-slate-800 flex flex-wrap items-center gap-2 text-xs text-slate-500">
            <span>Genesis block_id:</span>
            <HashValue
              value={alloc.genesis_block_id}
              href={routes.block(alloc.genesis_block_id)}
              truncate={false}
              label="genesis_block_id"
            />
          </div>
        </CardContent>
      </Card>

      {/* 3 Bigint Stat Cards: Allocated, Still Unspent, Moved */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card>
          <CardContent className="p-5 space-y-1">
            <div className="text-xs text-slate-500 dark:text-slate-400">
              {t('allocationStatAllocated')}
            </div>
            <div className="text-lg font-bold text-slate-900 dark:text-slate-100">
              <HsisAmount quanta={alloc.allocated} />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-5 space-y-1">
            <div className="text-xs text-slate-500 dark:text-slate-400">
              {t('allocationStatUnspent')}
            </div>
            <div className="text-lg font-bold text-emerald-700 dark:text-emerald-400">
              <HsisAmount quanta={alloc.unspent} />
            </div>
            <div className="text-[11px] font-mono text-slate-500">
              {unspentPct}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-5 space-y-1">
            <div className="text-xs text-slate-500 dark:text-slate-400">
              {t('allocationStatMoved')}
            </div>
            <div className="text-lg font-bold text-amber-700 dark:text-amber-400">
              <HsisAmount quanta={movedQuanta} />
            </div>
            <div className="text-[11px] font-mono text-slate-500">{movedPct}</div>
          </CardContent>
        </Card>
      </div>

      {/* Moved Progress Bar */}
      <Card>
        <CardContent className="p-5 space-y-2.5">
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-1 text-xs">
            <span className="font-semibold text-slate-900 dark:text-slate-100">
              {t('allocationMovedProgressTitle')}
            </span>
            <span className="font-mono text-slate-600 dark:text-slate-300 tabular-nums">
              {t('allocationMovedPercentLabel', {
                percent: movedPct,
                unspentPercent: unspentPct,
              })}
            </span>
          </div>
          <Progress
            basisPoints={movedBps}
            label={t('allocationMovedProgressTitle')}
            indicatorClassName="bg-amber-500 dark:bg-amber-400"
          />
        </CardContent>
      </Card>

      {/* Genesis Outputs Table */}
      <Card>
        <CardHeader>
          <CardTitle>
            {t('allocationOutputsTitle')} ({formatInt(alloc.outputs.length, lang)})
          </CardTitle>
          <CardDescription>{t('allocationOutputsSubtitle')}</CardDescription>
        </CardHeader>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>{t('colOutpoint')}</TableHead>
              <TableHead>{t('colAddress')}</TableHead>
              <TableHead className="text-right">{t('colAmount')}</TableHead>
              <TableHead className="text-right">{t('colSpendStatus')}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {alloc.outputs.map((out) => (
              <TableRow key={`${out.txid}:${out.index}`}>
                <TableCell className="font-mono whitespace-nowrap">
                  <HashValue
                    value={out.txid}
                    href={routes.tx(out.txid)}
                    startChars={10}
                    endChars={8}
                    label="genesis txid"
                  />
                  <span className="ml-1 text-slate-500">:{out.index}</span>
                </TableCell>
                <TableCell>
                  <HashValue
                    value={out.address}
                    href={routes.address(out.address)}
                    startChars={14}
                    endChars={10}
                    label="address"
                  />
                </TableCell>
                <TableCell className="text-right font-semibold">
                  <HsisAmount quanta={out.amount} />
                </TableCell>
                <TableCell className="text-right">
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
                    <span className="text-emerald-700 dark:text-emerald-400 font-semibold">
                      {t('txUnspentBadge')}
                    </span>
                  )}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </Card>

      {/* Recipient Address Cards & History */}
      <div className="space-y-4">
        <div>
          <h2 className="text-base font-bold text-slate-900 dark:text-slate-100">
            {t('allocationAddressesTitle')}
          </h2>
          <p className="text-xs text-slate-500 dark:text-slate-400">
            {t('allocationAddressesSubtitle')}
          </p>
        </div>

        {alloc.addresses.map((addr) => (
          <Card key={addr.address}>
            <CardHeader className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
              <div className="space-y-1">
                <HashValue
                  value={addr.address}
                  href={routes.address(addr.address)}
                  truncate={false}
                  label="recipient address"
                />
                <div className="flex flex-wrap items-center gap-3 text-xs text-slate-500">
                  <span>
                    {t('addressUtxoCountLabel')}:{' '}
                    <strong className="font-mono text-slate-800 dark:text-slate-200">
                      {formatInt(addr.utxo_count, lang)}
                    </strong>
                  </span>
                  <span aria-hidden="true">·</span>
                  <span>
                    {t('addressTxCountLabel')}:{' '}
                    <strong className="font-mono text-slate-800 dark:text-slate-200">
                      {formatInt(addr.tx_count, lang)}
                    </strong>
                  </span>
                </div>
              </div>

              <div className="text-left sm:text-right">
                <div className="text-[11px] text-slate-500">
                  {t('addressBalanceLabel')}
                </div>
                <div className="text-base font-bold text-teal-700 dark:text-teal-400">
                  <HsisAmount quanta={addr.balance} />
                </div>
              </div>
            </CardHeader>

            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>{t('colTxid')}</TableHead>
                  <TableHead>{t('colHeight')}</TableHead>
                  <TableHead className="text-right">{t('colReceived')}</TableHead>
                  <TableHead className="text-right">{t('colSent')}</TableHead>
                  <TableHead className="text-right">{t('colStatus')}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {addr.txs.map((tx) => {
                  const recQ = parseQuanta(tx.received) ?? 0n;
                  const sentQ = parseQuanta(tx.sent) ?? 0n;
                  return (
                    <TableRow key={tx.txid}>
                      <TableCell>
                        <HashValue
                          value={tx.txid}
                          href={routes.tx(tx.txid)}
                          startChars={12}
                          endChars={10}
                          label="txid"
                        />
                      </TableCell>
                      <TableCell className="font-mono tabular-nums">
                        <Link
                          href={routes.block(tx.height)}
                          className="text-teal-700 dark:text-teal-400 hover:underline"
                        >
                          #{formatInt(tx.height, lang)}
                        </Link>
                      </TableCell>
                      <TableCell className="text-right">
                        {recQ > 0n ? (
                          <HsisAmount
                            quanta={tx.received}
                            signed="plus"
                            className="text-emerald-700 dark:text-emerald-400 font-medium"
                          />
                        ) : (
                          <span className="font-mono text-slate-400">0 HSIS</span>
                        )}
                      </TableCell>
                      <TableCell className="text-right">
                        {sentQ > 0n ? (
                          <HsisAmount
                            quanta={tx.sent}
                            signed="minus"
                            className="text-amber-700 dark:text-amber-400 font-medium"
                          />
                        ) : (
                          <span className="font-mono text-slate-400">0 HSIS</span>
                        )}
                      </TableCell>
                      <TableCell className="text-right">
                        <ConfirmationsBadge confirmations={tx.confirmations} />
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          </Card>
        ))}
      </div>
    </div>
  );
}

export default function AllocationPage() {
  return (
    <Suspense fallback={<TableSkeleton rows={10} />}>
      <AllocationPageContent />
    </Suspense>
  );
}
