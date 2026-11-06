'use client';

import React, { useCallback } from 'react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import {
  AlertOctagon,
  ArrowRight,
  CheckCircle2,
  RefreshCw,
  Search,
} from 'lucide-react';
import { useApp } from './providers';
import { usePoll } from '@/hooks/use-poll';
import {
  fetchBlocks,
  fetchStatus,
  fetchSupply,
  type ApiResult,
  type BlockList,
  type Status,
  type SupplyInfo,
} from '@/lib/api';
import {
  computeBasisPoints,
  formatBasisPointsPercent,
  formatZvrg,
  formatInt,
  MINING_SUPPLY_QUANTA,
} from '@/lib/amount';
import { routes } from '@/lib/route';
import { formatRelativeTime, formatUtcTimestamp } from '@/lib/time';
import {
  Alert,
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Input,
  Progress,
  Skeleton,
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
  RecentBlocksCharts,
  TableSkeleton,
} from '@/components/explorer/shared';

interface DashboardBundle {
  status: Status;
  supply: SupplyInfo;
  blocks: BlockList;
}

export default function DashboardPage() {
  const { lang, t } = useApp();
  const router = useRouter();
  const [heroQuery, setHeroQuery] = React.useState('');

  const loadDashboard = useCallback(async (): Promise<ApiResult<DashboardBundle>> => {
    const [statusRes, supplyRes, blocksRes] = await Promise.all([
      fetchStatus(),
      fetchSupply(),
      fetchBlocks(),
    ]);

    if (!statusRes.ok) return statusRes;
    if (!supplyRes.ok) return supplyRes;
    if (!blocksRes.ok) return blocksRes;

    return {
      ok: true,
      data: {
        status: statusRes.data,
        supply: supplyRes.data,
        blocks: blocksRes.data,
      },
    };
  }, []);

  const { data, failure, isLoading, isRefreshing, nowUnix, reload } = usePoll(
    loadDashboard,
    []
  );

  const handleHeroSearch = (e: React.FormEvent) => {
    e.preventDefault();
    const q = heroQuery.trim();
    if (!q) return;
    router.push(routes.search(q));
  };

  const status = data?.status;
  const supply = data?.supply;
  const blocks = data?.blocks.blocks ?? [];

  const minedBps = supply
    ? computeBasisPoints(supply.mined_total, MINING_SUPPLY_QUANTA)
    : 0n;
  const minedPercentLabel = supply
    ? formatBasisPointsPercent(supply.mined_total, MINING_SUPPLY_QUANTA, lang)
    : '0%';

  return (
    <div className="space-y-6">
      {/* Header & Search Bar */}
      <div className="flex flex-col lg:flex-row lg:items-end justify-between gap-4 border-b border-slate-200 dark:border-slate-800/80 pb-5">
        <div className="space-y-1">
          <div className="flex items-center gap-2.5 text-xs text-slate-500 dark:text-slate-400">
            <span>Zevrigo Coin (ZVRG)</span>
            <span aria-hidden="true">·</span>
            <span className="inline-flex items-center gap-1.5 font-mono text-[11px] text-teal-700 dark:text-teal-400">
              <RefreshCw
                className={`h-3 w-3 ${isRefreshing ? 'animate-spin' : ''}`}
                aria-hidden="true"
              />
              <span>{t('liveUpdating')}</span>
            </span>
          </div>
          <h1 className="text-xl sm:text-2xl font-bold tracking-tight text-slate-900 dark:text-slate-100">
            {t('dashboardHeroTitle')}
          </h1>
          <p className="text-xs text-slate-600 dark:text-slate-400 max-w-2xl">
            {t('dashboardHeroSubtitle')}
          </p>
        </div>

        <form
          onSubmit={handleHeroSearch}
          className="flex w-full lg:w-96 items-center gap-2 shrink-0"
        >
          <div className="relative flex-1">
            <Search className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-slate-400" />
            <Input
              value={heroQuery}
              onChange={(e) => setHeroQuery(e.target.value)}
              placeholder={t('searchTriggerLabel')}
              aria-label={t('searchTriggerLabel')}
              className="pl-8 font-mono text-xs"
            />
          </div>
          <Button type="submit">{t('searchButton')}</Button>
        </form>
      </div>

      {/* Error or Not-Ready state if initial load failed */}
      {failure && !data && <ApiStateView failure={failure} onRetry={reload} />}

      {/* 8 Stat Cards Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3.5">
        {/* 1. Indexed Height */}
        <Card>
          <CardContent className="p-4 space-y-1">
            <div className="text-xs text-slate-500 dark:text-slate-400">
              {t('statIndexedHeight')}
            </div>
            {isLoading && !status ? (
              <Skeleton className="h-7 w-28" />
            ) : (
              <div className="text-lg font-bold font-mono tabular-nums text-slate-900 dark:text-slate-100">
                {status?.indexed_height !== null && status?.indexed_height !== undefined
                  ? `#${formatInt(status.indexed_height, lang)}`
                  : '—'}
              </div>
            )}
            <div className="text-[11px] font-mono text-slate-500 dark:text-slate-400 truncate">
              {status?.indexed_block_id ? (
                <HashValue
                  value={status.indexed_block_id}
                  href={routes.block(status.indexed_block_id)}
                  startChars={8}
                  endChars={6}
                  label="block_id"
                />
              ) : (
                '—'
              )}
            </div>
          </CardContent>
        </Card>

        {/* 2. Node Height & Sync State */}
        <Card>
          <CardContent className="p-4 space-y-1">
            <div className="text-xs text-slate-500 dark:text-slate-400">
              {t('statNodeHeight')}
            </div>
            {isLoading && !status ? (
              <Skeleton className="h-7 w-28" />
            ) : (
              <div className="text-lg font-bold font-mono tabular-nums text-slate-900 dark:text-slate-100">
                {status?.node_height !== null && status?.node_height !== undefined
                  ? `#${formatInt(status.node_height, lang)}`
                  : '—'}
              </div>
            )}
            <div className="text-[11px] text-emerald-700 dark:text-emerald-400 font-medium">
              {status &&
              status.indexed_height !== null &&
              status.node_height !== null &&
              status.indexed_height >= status.node_height
                ? t('statNodeSynced')
                : status &&
                  status.indexed_height !== null &&
                  status.node_height !== null
                ? t('bannerSyncingText', {
                    indexed: formatInt(status.indexed_height, lang),
                    node: formatInt(status.node_height, lang),
                  })
                : '—'}
            </div>
          </CardContent>
        </Card>

        {/* 3. Mempool Transactions */}
        <Card>
          <CardContent className="p-4 space-y-1">
            <div className="text-xs text-slate-500 dark:text-slate-400">
              {t('statMempoolTxs')}
            </div>
            {isLoading && !status ? (
              <Skeleton className="h-7 w-20" />
            ) : (
              <div className="text-lg font-bold font-mono tabular-nums text-slate-900 dark:text-slate-100">
                {status ? formatInt(status.mempool_count, lang) : '—'}
              </div>
            )}
            <div className="text-[11px]">
              <Link
                href={routes.mempool()}
                className="text-teal-700 dark:text-teal-400 hover:underline font-medium"
              >
                {t('navMempool')} →
              </Link>
            </div>
          </CardContent>
        </Card>

        {/* 4. Average Block Time */}
        <Card>
          <CardContent className="p-4 space-y-1">
            <div className="text-xs text-slate-500 dark:text-slate-400">
              {t('statAvgBlockTime')}
            </div>
            {isLoading && !supply ? (
              <Skeleton className="h-7 w-24" />
            ) : (
              <div className="text-lg font-bold font-mono tabular-nums text-slate-900 dark:text-slate-100">
                {supply?.average_block_seconds !== null &&
                supply?.average_block_seconds !== undefined
                  ? `${formatInt(Math.round(supply.average_block_seconds), lang)} s`
                  : '—'}
              </div>
            )}
            <div className="text-[11px] text-slate-500 dark:text-slate-400 font-mono">
              {t('statTargetBlockTime')}
            </div>
          </CardContent>
        </Card>

        {/* 5. Mined Supply */}
        <Card>
          <CardContent className="p-4 space-y-1">
            <div className="text-xs text-slate-500 dark:text-slate-400">
              {t('statMinedSupply')}
            </div>
            {isLoading && !supply ? (
              <Skeleton className="h-7 w-36" />
            ) : (
              <div className="text-base font-bold text-slate-900 dark:text-slate-100 truncate">
                {supply ? (
                  <ZvrgAmount quanta={supply.mined_total} compact maxDecimals={6} />
                ) : (
                  '—'
                )}
              </div>
            )}
            <div className="text-[11px] text-slate-500 dark:text-slate-400 font-mono tabular-nums">
              {minedPercentLabel} / 43.8M ZVRG
            </div>
          </CardContent>
        </Card>

        {/* 6. Cumulative Fees */}
        <Card>
          <CardContent className="p-4 space-y-1">
            <div className="text-xs text-slate-500 dark:text-slate-400">
              {t('supplyCumulativeFees')}
            </div>
            {isLoading && !supply ? (
              <Skeleton className="h-7 w-36" />
            ) : (
              <div className="text-base font-bold text-slate-900 dark:text-slate-100 truncate">
                {supply ? (
                  <ZvrgAmount quanta={supply.fees_total} compact maxDecimals={6} />
                ) : (
                  '—'
                )}
              </div>
            )}
            <div className="text-[11px]">
              <Link
                href={routes.supply()}
                className="text-teal-700 dark:text-teal-400 hover:underline font-medium"
              >
                {t('navSupply')} →
              </Link>
            </div>
          </CardContent>
        </Card>

        {/* 7. Next Block Subsidy */}
        <Card>
          <CardContent className="p-4 space-y-1">
            <div className="text-xs text-slate-500 dark:text-slate-400">
              {t('statNextSubsidy')}
            </div>
            {isLoading && !supply ? (
              <Skeleton className="h-7 w-36" />
            ) : (
              <div className="text-base font-bold text-slate-900 dark:text-slate-100 truncate">
                {supply ? (
                  <ZvrgAmount quanta={supply.next_subsidy} compact maxDecimals={6} />
                ) : (
                  '—'
                )}
              </div>
            )}
            <div className="text-[11px] text-slate-500 dark:text-slate-400 font-mono">
              Slow start (#{supply ? formatInt(supply.height + 1, lang) : '—'} / 10,080)
            </div>
          </CardContent>
        </Card>

        {/* 8. Chain Tip Target */}
        <Card>
          <CardContent className="p-4 space-y-1">
            <div className="text-xs text-slate-500 dark:text-slate-400">
              {t('statTipTarget')}
            </div>
            {isLoading && !supply ? (
              <Skeleton className="h-7 w-28" />
            ) : (
              <div className="text-lg font-bold font-mono tabular-nums text-slate-900 dark:text-slate-100">
                0x{supply?.tip_target ?? '—'}
              </div>
            )}
            <div className="text-[11px] text-slate-500 dark:text-slate-400 font-mono truncate">
              ASERT (720-block half-life)
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Supply Progress & Schedule Verification Card */}
      {supply && (
        <Card>
          <CardContent className="p-5 space-y-3">
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
              <div>
                <h2 className="text-sm font-semibold text-slate-900 dark:text-slate-100">
                  {t('supplyProgressTitle')}
                </h2>
                <p className="text-xs text-slate-500 dark:text-slate-400">
                  {t('supplyProgressSubtitle')}
                </p>
              </div>

              {supply.matches_schedule ? (
                <div className="inline-flex items-center gap-1.5 text-xs font-semibold text-emerald-700 dark:text-emerald-400">
                  <CheckCircle2 className="h-4 w-4 shrink-0" />
                  <span>{t('supplyVerificationValid')}</span>
                </div>
              ) : (
                <Alert variant="destructive" className="py-2 px-3 w-auto">
                  <div className="inline-flex items-center gap-2 font-semibold">
                    <AlertOctagon className="h-4 w-4 shrink-0" />
                    <span>{t('supplyVerificationInvalid')}</span>
                  </div>
                </Alert>
              )}
            </div>

            <Progress
              basisPoints={minedBps}
              nonZero={supply.mined_total !== '0'}
              label={t('supplyProgressTitle')}
            />

            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 text-xs text-slate-600 dark:text-slate-300 font-mono tabular-nums">
              <div>
                <ZvrgAmount quanta={supply.mined_total} /> /{' '}
                {formatZvrg(MINING_SUPPLY_QUANTA, lang)} ({minedPercentLabel})
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Charts from the latest 20 blocks */}
      {blocks.length > 0 && <RecentBlocksCharts blocks={blocks} />}

      {/* Latest Blocks Table */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between gap-4">
          <div>
            <CardTitle>{t('latestBlocksTitle')}</CardTitle>
            <CardDescription>{t('latestBlocksSubtitle')}</CardDescription>
          </div>
          <Button variant="outline" size="sm" asChild>
            <Link href={routes.blocks()}>
              <span>{t('viewAllBlocks')}</span>
              <ArrowRight className="h-3.5 w-3.5" />
            </Link>
          </Button>
        </CardHeader>

        {isLoading && blocks.length === 0 ? (
          <TableSkeleton rows={8} />
        ) : (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{t('colHeight')}</TableHead>
                <TableHead>{t('colBlockId')}</TableHead>
                <TableHead>{t('colTime')}</TableHead>
                <TableHead className="text-right">{t('colTxs')}</TableHead>
                <TableHead className="text-right">{t('colSize')}</TableHead>
                <TableHead className="text-right">{t('colStatus')}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {blocks.map((b) => (
                <TableRow key={b.block_id}>
                  <TableCell className="font-mono font-semibold tabular-nums">
                    <Link
                      href={routes.block(b.height)}
                      className="text-teal-700 dark:text-teal-400 hover:underline"
                    >
                      #{formatInt(b.height, lang)}
                    </Link>
                  </TableCell>
                  <TableCell>
                    <HashValue
                      value={b.block_id}
                      href={routes.block(b.block_id)}
                      startChars={10}
                      endChars={8}
                      label="block_id"
                    />
                  </TableCell>
                  <TableCell className="whitespace-nowrap">
                    <div className="font-mono text-xs tabular-nums text-slate-800 dark:text-slate-200">
                      {formatUtcTimestamp(b.timestamp)}
                    </div>
                    <div className="text-[11px] text-slate-500 dark:text-slate-400">
                      {formatRelativeTime(b.timestamp, nowUnix, lang)}
                    </div>
                  </TableCell>
                  <TableCell className="text-right font-mono tabular-nums">
                    {formatInt(b.tx_count, lang)}
                  </TableCell>
                  <TableCell className="text-right font-mono tabular-nums">
                    {formatInt(b.size, lang)}
                  </TableCell>
                  <TableCell className="text-right">
                    <ConfirmationsBadge confirmations={b.confirmations} />
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
