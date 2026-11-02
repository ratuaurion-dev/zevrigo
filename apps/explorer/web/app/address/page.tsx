'use client';

import React, { Suspense, useCallback, useEffect, useState } from 'react';
import Link from 'next/link';
import { useSearchParams } from 'next/navigation';
import QRCode from 'qrcode';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import { useApp } from '@/app/providers';
import { usePoll } from '@/hooks/use-poll';
import { fetchAddress } from '@/lib/api';
import { formatInt, parseQuanta } from '@/lib/amount';
import { isValidAddress, isValidPage, routes } from '@/lib/route';
import {
  Button,
  Card,
  CardContent,
  CardDescription,
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
  HsisAmount,
  TableSkeleton,
} from '@/components/explorer/shared';

function AddressDetailContent() {
  const { lang, t } = useApp();
  const searchParams = useSearchParams();
  const rawAddress = (searchParams.get('a') ?? '').trim().toLowerCase();
  const rawPage = searchParams.get('page') ?? '0';

  const validAddress = isValidAddress(rawAddress);
  const validPage = isValidPage(rawPage);
  const pageNum = validPage ? Number(rawPage) : 0;

  const [qrDataUrl, setQrDataUrl] = useState<string | null>(null);

  // Generate QR code locally in the browser with zero external requests
  useEffect(() => {
    if (!validAddress) {
      setQrDataUrl(null);
      return;
    }
    let cancelled = false;
    QRCode.toDataURL(rawAddress, {
      errorCorrectionLevel: 'M',
      margin: 1,
      width: 160,
      color: {
        dark: '#0f172a',
        light: '#ffffff',
      },
    })
      .then((url) => {
        if (!cancelled) setQrDataUrl(url);
      })
      .catch(() => {
        if (!cancelled) setQrDataUrl(null);
      });
    return () => {
      cancelled = true;
    };
  }, [rawAddress, validAddress]);

  const fetcher = useCallback(() => {
    if (!validAddress || !validPage) {
      return Promise.resolve({
        ok: false as const,
        failure: { kind: 'http' as const, status: 400, error: 'bad_request' },
      });
    }
    return fetchAddress(rawAddress, pageNum);
  }, [rawAddress, pageNum, validAddress, validPage]);

  const { data: info, failure, isLoading, reload } = usePoll(fetcher, [
    rawAddress,
    pageNum,
    validAddress,
    validPage,
  ]);

  if (!validAddress || !validPage) {
    return (
      <ApiStateView
        failure={{ kind: 'http', status: 400, error: 'bad_request' }}
      />
    );
  }

  if (failure && !info) {
    return <ApiStateView failure={failure} onRetry={reload} />;
  }

  if (isLoading && !info) {
    return <TableSkeleton rows={10} />;
  }

  if (!info) return null;

  const totalPages = Math.max(1, Math.ceil(info.tx_count / info.page_size));
  const hasPrev = info.page > 0;
  const hasNext = info.page + 1 < totalPages;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="border-b border-slate-200 dark:border-slate-800 pb-4 space-y-1">
        <h1 className="text-xl font-bold text-slate-900 dark:text-slate-100">
          {t('addressDetailTitle')}
        </h1>
        <HashValue value={info.address} truncate={false} label="address" />
      </div>

      {/* Summary & Local QR Code Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-5">
        <Card className="lg:col-span-2">
          <CardContent className="p-5 space-y-5">
            <div className="space-y-1">
              <div className="text-xs text-slate-500 dark:text-slate-400">
                {t('addressBalanceLabel')}
              </div>
              <div className="text-2xl sm:text-3xl font-bold text-teal-700 dark:text-teal-400">
                <HsisAmount quanta={info.balance} />
              </div>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 pt-4 border-t border-slate-100 dark:border-slate-800 text-xs">
              <div>
                <div className="text-slate-500 dark:text-slate-400">
                  {t('addressReceivedLabel')}
                </div>
                <div className="mt-0.5 font-semibold text-emerald-700 dark:text-emerald-400">
                  <HsisAmount quanta={info.received} signed="plus" />
                </div>
              </div>

              <div>
                <div className="text-slate-500 dark:text-slate-400">
                  {t('addressSentLabel')}
                </div>
                <div className="mt-0.5 font-semibold text-amber-700 dark:text-amber-400">
                  <HsisAmount quanta={info.sent} signed="minus" />
                </div>
              </div>

              <div>
                <div className="text-slate-500 dark:text-slate-400">
                  {t('addressUtxoCountLabel')}
                </div>
                <div className="mt-0.5 font-mono font-semibold tabular-nums">
                  {formatInt(info.utxo_count, lang)} UTXO
                </div>
              </div>

              <div>
                <div className="text-slate-500 dark:text-slate-400">
                  {t('addressTxCountLabel')}
                </div>
                <div className="mt-0.5 font-mono font-semibold tabular-nums">
                  {formatInt(info.tx_count, lang)}
                </div>
              </div>
            </div>

            <div className="pt-3 border-t border-slate-100 dark:border-slate-800 text-xs">
              <div className="text-slate-500 dark:text-slate-400 mb-1">
                {t('addressPayloadLabel')}
              </div>
              <div className="flex flex-wrap items-center gap-2 font-mono text-[11px]">
                <span>kind #{info.kind}</span>
                <span aria-hidden="true">·</span>
                <HashValue value={info.payload} truncate={false} label="payload" />
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Local QR Code Card */}
        <Card>
          <CardHeader>
            <CardTitle>{t('addressQrCodeTitle')}</CardTitle>
            <CardDescription>{t('addressQrCodeSubtitle')}</CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col items-center justify-center p-5">
            {qrDataUrl ? (
              <div className="rounded-lg border border-slate-200 bg-white p-3">
                <img
                  src={qrDataUrl}
                  alt={`QR ${info.address}`}
                  width={144}
                  height={144}
                  referrerPolicy="no-referrer"
                  className="h-36 w-36"
                />
              </div>
            ) : (
              <div className="h-36 w-36 rounded-lg bg-slate-100 dark:bg-slate-800 animate-pulse" />
            )}
          </CardContent>
        </Card>
      </div>

      {/* Transaction History Table */}
      <Card>
        <CardHeader className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
          <div>
            <CardTitle>
              {t('addressHistoryTitle')} ({formatInt(info.tx_count, lang)})
            </CardTitle>
            <CardDescription>
              {t('paginationPageInfo', {
                page: `${formatInt(info.page + 1, lang)} / ${formatInt(totalPages, lang)}`,
              })}
            </CardDescription>
          </div>

          <div className="flex items-center gap-2">
            {hasPrev ? (
              <Button variant="outline" size="sm" asChild>
                <Link href={routes.address(info.address, info.page - 1)}>
                  <ChevronLeft className="h-3.5 w-3.5" />
                  <span>{t('paginationPrev')}</span>
                </Link>
              </Button>
            ) : (
              <Button variant="outline" size="sm" disabled>
                <ChevronLeft className="h-3.5 w-3.5" />
                <span>{t('paginationPrev')}</span>
              </Button>
            )}

            {hasNext ? (
              <Button variant="outline" size="sm" asChild>
                <Link href={routes.address(info.address, info.page + 1)}>
                  <span>{t('paginationNext')}</span>
                  <ChevronRight className="h-3.5 w-3.5" />
                </Link>
              </Button>
            ) : (
              <Button variant="outline" size="sm" disabled>
                <span>{t('paginationNext')}</span>
                <ChevronRight className="h-3.5 w-3.5" />
              </Button>
            )}
          </div>
        </CardHeader>

        {info.txs.length === 0 ? (
          <div className="p-8 text-center text-xs text-slate-500">
            {t('addressHistoryEmpty')}
          </div>
        ) : (
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
              {info.txs.map((tx) => {
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
        )}
      </Card>
    </div>
  );
}

export default function AddressPage() {
  return (
    <Suspense fallback={<TableSkeleton rows={10} />}>
      <AddressDetailContent />
    </Suspense>
  );
}
