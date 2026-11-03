'use client';

import React, { useState } from 'react';
import Link from 'next/link';
import {
  AlertTriangle,
  Check,
  CheckCircle2,
  Clock,
  Copy,
  RefreshCw,
  ShieldAlert,
} from 'lucide-react';
import { toast } from 'sonner';
import { useApp } from '@/app/providers';
import type { ApiFailure, BlockRow } from '@/lib/api';
import {
  formatHsis,
  formatHsisCompact,
  formatInt,
  formatQuantaExact,
} from '@/lib/amount';
import { truncateMiddle } from '@/lib/route';
import { Alert, Button, Card, CardContent, Skeleton, Tooltip } from '@/components/ui/primitives';
import { cn } from '@/lib/utils';

/**
 * Custom SVG mark for Homeostasis Coin (HSIS):
 * A balanced equilibrium circle with dual harmonic counter-waves.
 */
export function HomeostasisLogo({ className = 'w-6 h-6' }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 32 32"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className={className}
      aria-hidden="true"
    >
      <circle
        cx="16"
        cy="16"
        r="13.5"
        className="stroke-teal-600 dark:stroke-teal-400"
        strokeWidth="2.2"
      />
      <path
        d="M5.5 16C9 10.5 12.5 10.5 16 16C19.5 21.5 23 21.5 26.5 16"
        className="stroke-teal-600 dark:stroke-teal-400"
        strokeWidth="2.2"
        strokeLinecap="round"
      />
      <path
        d="M7.5 16H24.5"
        className="stroke-slate-400 dark:stroke-slate-600"
        strokeWidth="1.2"
        strokeDasharray="2 2"
      />
      <circle cx="16" cy="16" r="2.2" className="fill-teal-600 dark:fill-teal-400" />
    </svg>
  );
}

/**
 * Confirmations indicator required everywhere a block or transaction is shown:
 * `< 6` -> amber ("not yet safe / belum aman")
 * `>= 6` -> emerald green
 * Hover tooltip explains: "This chain has no reorg limit; more confirmations = safer."
 */
export function ConfirmationsBadge({
  confirmations,
  className,
}: {
  confirmations: number;
  className?: string;
}) {
  const { lang, t } = useApp();
  const isSafe = confirmations >= 6;
  const countFormatted = formatInt(confirmations, lang);
  const countText =
    confirmations === 1
      ? t('confirmationsOne')
      : t('confirmationsCount', { count: countFormatted });

  return (
    <Tooltip content={t('confirmationsTooltip')}>
      <span
        className={cn(
          'inline-flex items-center gap-1.5 font-mono text-xs tabular-nums',
          isSafe
            ? 'text-emerald-700 dark:text-emerald-400'
            : 'text-amber-700 dark:text-amber-400',
          className
        )}
      >
        {isSafe ? (
          <CheckCircle2 className="h-3.5 w-3.5 shrink-0" aria-hidden="true" />
        ) : (
          <AlertTriangle className="h-3.5 w-3.5 shrink-0" aria-hidden="true" />
        )}
        <span>{countText}</span>
        {!isSafe && (
          <>
            <span aria-hidden="true">·</span>
            <span className="font-sans font-medium">{t('confirmationsUnsafe')}</span>
          </>
        )}
      </span>
    </Tooltip>
  );
}

/**
 * Copy button with Sonner toast feedback and accessible label.
 */
export function CopyButton({
  value,
  label = 'hash',
  className,
}: {
  value: string;
  label?: string;
  className?: string;
}) {
  const { t } = useApp();
  const [copied, setCopied] = useState(false);

  const handleCopy = async (e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await navigator.clipboard.writeText(value);
    } catch {
      // Fallback for restricted clipboard environments
      const textarea = document.createElement('textarea');
      textarea.value = value;
      textarea.style.position = 'fixed';
      textarea.style.opacity = '0';
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand('copy');
      document.body.removeChild(textarea);
    }
    setCopied(true);
    toast.success(t('copiedToast', { label }));
    setTimeout(() => setCopied(false), 1500);
  };

  return (
    <button
      type="button"
      onClick={handleCopy}
      aria-label={t('copyHash', { label })}
      title={t('copyHash', { label })}
      className={cn(
        'inline-flex h-6 w-6 items-center justify-center rounded text-slate-400 hover:bg-slate-100 hover:text-slate-700 dark:text-slate-500 dark:hover:bg-slate-800 dark:hover:text-slate-200 transition-colors cursor-pointer shrink-0',
        className
      )}
    >
      {copied ? (
        <Check className="h-3.5 w-3.5 text-emerald-600 dark:text-emerald-400" />
      ) : (
        <Copy className="h-3.5 w-3.5" />
      )}
    </button>
  );
}

/**
 * Monospace hash or address display with optional link, middle truncation, full value tooltip, and copy button.
 */
export function HashValue({
  value,
  href,
  truncate = true,
  startChars = 10,
  endChars = 8,
  label = 'hash',
  showCopy = true,
  className,
}: {
  value: string;
  href?: string;
  truncate?: boolean;
  startChars?: number;
  endChars?: number;
  label?: string;
  showCopy?: boolean;
  className?: string;
}) {
  const display = truncate ? truncateMiddle(value, startChars, endChars) : value;

  const inner = href ? (
    <Link
      href={href}
      className="font-mono text-xs text-teal-700 dark:text-teal-400 hover:underline break-all"
    >
      {display}
    </Link>
  ) : (
    <span className="font-mono text-xs text-slate-800 dark:text-slate-200 break-all">
      {display}
    </span>
  );

  return (
    <span className={cn('inline-flex items-center gap-1.5', className)}>
      {truncate ? <Tooltip content={value}>{inner}</Tooltip> : inner}
      {showCopy && <CopyButton value={value} label={label} />}
    </span>
  );
}

/**
 * Coin amount formatted with BigInt math + exact quanta tooltip on hover.
 */
export function HsisAmount({
  quanta,
  compact = false,
  maxDecimals = 6,
  signed = 'none',
  includeTicker = true,
  className,
}: {
  quanta: string | bigint;
  compact?: boolean;
  maxDecimals?: number;
  signed?: 'plus' | 'minus' | 'none';
  includeTicker?: boolean;
  className?: string;
}) {
  const { lang, t } = useApp();
  const exactQuanta = formatQuantaExact(quanta, lang);
  const fullFormatted = formatHsis(quanta, lang, { includeTicker: true, signed });
  const display = compact
    ? formatHsisCompact(quanta, lang, maxDecimals, includeTicker)
    : formatHsis(quanta, lang, { includeTicker, signed });

  return (
    <Tooltip content={`${fullFormatted} (${t('exactQuantaTooltip', { quanta: exactQuanta })})`}>
      <span className={cn('font-mono tabular-nums', className)}>{display}</span>
    </Tooltip>
  );
}

/**
 * Unified error / not-ready / not-found view for pages.
 */
export function ApiStateView({
  failure,
  onRetry,
}: {
  failure: ApiFailure;
  onRetry?: () => void;
}) {
  const { t } = useApp();

  if (failure.kind === 'http' && failure.status === 503) {
    return (
      <Alert variant="info" className="my-6 flex items-center justify-between gap-4">
        <div className="flex items-start gap-3">
          <Clock className="h-4 w-4 shrink-0 mt-0.5" />
          <div>
            <div className="font-semibold">{t('bannerNotReadyTitle')}</div>
            <p className="mt-0.5 text-slate-600 dark:text-slate-300">
              {t('bannerNotReadyBody')}
            </p>
          </div>
        </div>
        {onRetry && (
          <Button variant="outline" size="sm" onClick={onRetry}>
            <RefreshCw className="h-3.5 w-3.5" />
            {t('retryButton')}
          </Button>
        )}
      </Alert>
    );
  }

  if (failure.kind === 'http' && failure.status === 429) {
    return (
      <Alert variant="warning" className="my-6 flex items-center justify-between gap-4">
        <div className="flex items-start gap-3">
          <Clock className="h-4 w-4 shrink-0 mt-0.5" />
          <div>
            <div className="font-semibold">{t('rateLimitedTitle')}</div>
            <p className="mt-0.5">{t('rateLimitedBody')}</p>
          </div>
        </div>
        {onRetry && (
          <Button variant="outline" size="sm" onClick={onRetry}>
            <RefreshCw className="h-3.5 w-3.5" />
            {t('retryButton')}
          </Button>
        )}
      </Alert>
    );
  }

  if (failure.kind === 'http' && failure.status === 400) {
    return (
      <Alert variant="warning" className="my-6">
        <div className="flex items-start gap-3">
          <AlertTriangle className="h-4 w-4 shrink-0 mt-0.5" />
          <div>
            <div className="font-semibold">{t('invalidInputTitle')}</div>
            <p className="mt-0.5">{t('invalidInputBody')}</p>
          </div>
        </div>
      </Alert>
    );
  }

  if (failure.kind === 'http' && failure.status === 404) {
    return (
      <Alert variant="default" className="my-6">
        <div className="flex items-center justify-between gap-4">
          <div>
            <div className="font-semibold text-sm">{t('notFoundTitle')}</div>
            <p className="mt-1 text-slate-500 dark:text-slate-400">{t('notFoundBody')}</p>
          </div>
          <Button variant="outline" size="sm" asChild>
            <Link href="/">
              {t('backToDashboard')}
            </Link>
          </Button>
        </div>
      </Alert>
    );
  }

  return (
    <Alert variant="destructive" className="my-6">
      <div className="flex items-center justify-between gap-4">
        <div className="flex items-start gap-3">
          <ShieldAlert className="h-4 w-4 shrink-0 mt-0.5" />
          <div>
            <div className="font-semibold">{t('networkErrorTitle')}</div>
            <p className="mt-0.5">{t('networkErrorBody')}</p>
          </div>
        </div>
        {onRetry && (
          <Button variant="outline" size="sm" onClick={onRetry}>
            <RefreshCw className="h-3.5 w-3.5" />
            {t('retryButton')}
          </Button>
        )}
      </div>
    </Alert>
  );
}

/**
 * Clean SVG Bar Charts for the latest 20 blocks (interval in seconds, size in bytes, tx count).
 * Strictly uses integer block fields (never coin amounts).
 */
export function RecentBlocksCharts({ blocks }: { blocks: BlockRow[] }) {
  const { lang, t } = useApp();
  // Sort ascending by height so bars go oldest -> newest left-to-right
  const ordered = [...blocks].sort((a, b) => a.height - b.height);

  // Compute block intervals from consecutive timestamps
  const intervals: { height: number; value: number }[] = [];
  for (let i = 1; i < ordered.length; i++) {
    const diff = ordered[i].timestamp - ordered[i - 1].timestamp;
    intervals.push({
      height: ordered[i].height,
      value: diff > 0 ? diff : 0,
    });
  }

  const sizes = ordered.map((b) => ({ height: b.height, value: b.size }));
  const txCounts = ordered.map((b) => ({ height: b.height, value: b.tx_count }));

  const maxInterval = Math.max(180, ...intervals.map((d) => d.value), 1);
  const maxSize = Math.max(1000, ...sizes.map((d) => d.value), 1);
  const maxTxs = Math.max(5, ...txCounts.map((d) => d.value), 1);

  return (
    <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
      {/* 1. Block Interval Chart */}
      <Card>
        <div className="px-5 pt-4 pb-2 flex items-baseline justify-between gap-2">
          <div>
            <h3 className="text-xs font-semibold text-slate-900 dark:text-slate-100">
              {t('chartIntervalTitle')}
            </h3>
            <p className="text-[11px] text-slate-500 dark:text-slate-400">
              {t('chartIntervalSubtitle')}
            </p>
          </div>
          <span className="font-mono text-xs text-teal-700 dark:text-teal-400 tabular-nums shrink-0">
            120s
          </span>
        </div>
        <CardContent className="pt-2 pb-4">
          <div className="h-28 flex items-end gap-1 pt-4 border-b border-slate-100 dark:border-slate-800">
            {intervals.map((item) => {
              const pct = Math.max(8, Math.round((item.value / maxInterval) * 100));
              return (
                <Tooltip
                  key={item.height}
                  content={`#${formatInt(item.height, lang)}: ${formatInt(item.value, lang)}s`}
                  className="flex-1 h-full flex items-end"
                >
                  <Link
                    href={`/block/?id=${item.height}`}
                    aria-label={`Block ${item.height}: ${item.value}s`}
                    className="w-full rounded-t-xs bg-teal-600/80 hover:bg-teal-500 dark:bg-teal-500/80 dark:hover:bg-teal-400 transition-colors"
                    style={{ height: `${pct}%` }}
                  />
                </Tooltip>
              );
            })}
          </div>
          <div className="mt-1.5 flex justify-between text-[10px] font-mono text-slate-400 tabular-nums">
            <span>#{ordered[1]?.height ?? 0}</span>
            <span>#{ordered[ordered.length - 1]?.height ?? 0}</span>
          </div>
        </CardContent>
      </Card>

      {/* 2. Block Size Chart */}
      <Card>
        <div className="px-5 pt-4 pb-2 flex items-baseline justify-between gap-2">
          <div>
            <h3 className="text-xs font-semibold text-slate-900 dark:text-slate-100">
              {t('chartSizeTitle')}
            </h3>
            <p className="text-[11px] text-slate-500 dark:text-slate-400">
              {t('chartSizeSubtitle')}
            </p>
          </div>
          <span className="font-mono text-xs text-slate-600 dark:text-slate-300 tabular-nums shrink-0">
            {formatInt(ordered[ordered.length - 1]?.size ?? 0, lang)} B
          </span>
        </div>
        <CardContent className="pt-2 pb-4">
          <div className="h-28 flex items-end gap-1 pt-4 border-b border-slate-100 dark:border-slate-800">
            {sizes.map((item) => {
              const pct = Math.max(8, Math.round((item.value / maxSize) * 100));
              return (
                <Tooltip
                  key={item.height}
                  content={`#${formatInt(item.height, lang)}: ${formatInt(item.value, lang)} B`}
                  className="flex-1 h-full flex items-end"
                >
                  <Link
                    href={`/block/?id=${item.height}`}
                    aria-label={`Block ${item.height} size: ${item.value} bytes`}
                    className="w-full rounded-t-xs bg-sky-600/80 hover:bg-sky-500 dark:bg-sky-500/80 dark:hover:bg-sky-400 transition-colors"
                    style={{ height: `${pct}%` }}
                  />
                </Tooltip>
              );
            })}
          </div>
          <div className="mt-1.5 flex justify-between text-[10px] font-mono text-slate-400 tabular-nums">
            <span>#{ordered[0]?.height ?? 0}</span>
            <span>#{ordered[ordered.length - 1]?.height ?? 0}</span>
          </div>
        </CardContent>
      </Card>

      {/* 3. Transactions per Block Chart */}
      <Card>
        <div className="px-5 pt-4 pb-2 flex items-baseline justify-between gap-2">
          <div>
            <h3 className="text-xs font-semibold text-slate-900 dark:text-slate-100">
              {t('chartTxCountTitle')}
            </h3>
            <p className="text-[11px] text-slate-500 dark:text-slate-400">
              {t('chartTxCountSubtitle')}
            </p>
          </div>
          <span className="font-mono text-xs text-slate-600 dark:text-slate-300 tabular-nums shrink-0">
            {formatInt(ordered[ordered.length - 1]?.tx_count ?? 0, lang)} tx
          </span>
        </div>
        <CardContent className="pt-2 pb-4">
          <div className="h-28 flex items-end gap-1 pt-4 border-b border-slate-100 dark:border-slate-800">
            {txCounts.map((item) => {
              const pct = Math.max(12, Math.round((item.value / maxTxs) * 100));
              return (
                <Tooltip
                  key={item.height}
                  content={`#${formatInt(item.height, lang)}: ${formatInt(item.value, lang)} tx`}
                  className="flex-1 h-full flex items-end"
                >
                  <Link
                    href={`/block/?id=${item.height}`}
                    aria-label={`Block ${item.height} transactions: ${item.value}`}
                    className={cn(
                      'w-full rounded-t-xs transition-colors',
                      item.value > 1
                        ? 'bg-amber-500 hover:bg-amber-400 dark:bg-amber-400'
                        : 'bg-slate-400/70 hover:bg-slate-500 dark:bg-slate-600 dark:hover:bg-slate-500'
                    )}
                    style={{ height: `${pct}%` }}
                  />
                </Tooltip>
              );
            })}
          </div>
          <div className="mt-1.5 flex justify-between text-[10px] font-mono text-slate-400 tabular-nums">
            <span>#{ordered[0]?.height ?? 0}</span>
            <span>#{ordered[ordered.length - 1]?.height ?? 0}</span>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

export function TableSkeleton({ rows = 8 }: { rows?: number }) {
  return (
    <div className="space-y-2.5 p-5">
      {Array.from({ length: rows }).map((_, i) => (
        <Skeleton key={i} className="h-8 w-full" />
      ))}
    </div>
  );
}
