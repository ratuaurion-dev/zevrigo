'use client';

import React, { useCallback, useState } from 'react';
import { AlertTriangle, CheckCircle2, Droplets, RefreshCw, ShieldCheck } from 'lucide-react';
import { useApp } from '@/app/providers';
import { usePoll } from '@/hooks/use-poll';
import type { ApiFailure } from '@/lib/api';
import { formatInt } from '@/lib/amount';
import {
  fetchFaucetStatus,
  faucetErrorKey,
  isRegtestAddress,
  requestFaucet,
  type FaucetPayout,
} from '@/lib/faucet';
import { routes } from '@/lib/route';
import {
  Alert,
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Input,
} from '@/components/ui/primitives';
import { HashValue, TableSkeleton, ZvrgAmount } from '@/components/explorer/shared';

type SubmitState =
  | { kind: 'idle' }
  | { kind: 'sending' }
  | { kind: 'sent'; payout: FaucetPayout }
  | { kind: 'failed'; failure: ApiFailure };

export default function FaucetPage() {
  const { lang, t } = useApp();
  const fetcher = useCallback(() => fetchFaucetStatus(), []);
  const { data: status, failure, isLoading, isRefreshing, reload } = usePoll(fetcher, []);
  const [address, setAddress] = useState('');
  const [submit, setSubmit] = useState<SubmitState>({ kind: 'idle' });

  const addressLooksValid = isRegtestAddress(address);
  const showAddressHint = address.trim().length > 0 && !addressLooksValid;

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    if (!addressLooksValid || submit.kind === 'sending') {
      return;
    }
    setSubmit({ kind: 'sending' });
    const result = await requestFaucet(address);
    if (result.ok) {
      setSubmit({ kind: 'sent', payout: result.data });
      setAddress('');
    } else {
      setSubmit({ kind: 'failed', failure: result.failure });
    }
    reload();
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-slate-200 dark:border-slate-800 pb-4">
        <div className="space-y-1">
          <h1 className="text-xl sm:text-2xl font-bold text-slate-900 dark:text-slate-100">
            {t('faucetPageTitle')}
          </h1>
          <p className="text-xs text-slate-500 dark:text-slate-400">{t('faucetPageSubtitle')}</p>
        </div>
        <div className="inline-flex items-center gap-2 font-mono text-xs text-teal-700 dark:text-teal-400">
          <RefreshCw className={`h-3.5 w-3.5 ${isRefreshing ? 'animate-spin' : ''}`} />
          <span>{t('liveUpdating')}</span>
        </div>
      </div>

      <Alert variant="warning" className="flex items-start gap-3">
        <AlertTriangle className="h-4 w-4 shrink-0 mt-0.5" />
        <p className="text-xs leading-relaxed">{t('faucetDemoNotice')}</p>
      </Alert>

      {/* Faucet status */}
      {isLoading && !status ? (
        <TableSkeleton rows={3} />
      ) : !status ? (
        <Alert variant="destructive" className="space-y-3">
          <div className="flex items-center gap-2.5 font-semibold">
            <AlertTriangle className="h-4 w-4 shrink-0" />
            <span>{t('faucetUnavailableTitle')}</span>
          </div>
          <p className="text-xs leading-relaxed">
            {failure ? t(faucetErrorKey(failure)) : t('faucetErrUnavailable')}
          </p>
          <Button variant="outline" size="sm" onClick={reload}>
            {t('faucetRetry')}
          </Button>
        </Alert>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <Card>
            <CardContent className="p-4 space-y-1">
              <div className="text-xs text-slate-500 dark:text-slate-400">{t('faucetAmountLabel')}</div>
              <div className="text-base font-bold text-slate-900 dark:text-slate-100">
                <ZvrgAmount quanta={status.amount} />
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="p-4 space-y-1">
              <div className="text-xs text-slate-500 dark:text-slate-400">{t('faucetRemainingLabel')}</div>
              <div className="text-base font-bold font-mono tabular-nums text-slate-900 dark:text-slate-100">
                {t('faucetRemainingValue', {
                  remaining: formatInt(status.remaining_today, lang),
                  limit: formatInt(status.daily_limit, lang),
                })}
              </div>
              <div className="text-[11px] text-slate-500">{t('faucetResetNote')}</div>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="p-4 space-y-1">
              <div className="text-xs text-slate-500 dark:text-slate-400">{t('faucetBalanceLabel')}</div>
              <div className="text-base font-bold text-slate-900 dark:text-slate-100">
                <ZvrgAmount quanta={status.balance} />
              </div>
              {!status.ready && (
                <div className="text-[11px] text-amber-700 dark:text-amber-400">
                  {status.synced ? t('faucetErrEmpty') : t('faucetErrBusy')}
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      )}

      {/* Request form */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-sm">
            <Droplets className="h-4 w-4 text-teal-600 dark:text-teal-400" />
            {t('faucetFormTitle')}
          </CardTitle>
          <CardDescription className="text-xs">{t('faucetAddressHint')}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <form onSubmit={handleSubmit} className="flex flex-col sm:flex-row gap-2">
            <label htmlFor="faucet-address" className="sr-only">
              {t('faucetAddressLabel')}
            </label>
            <Input
              id="faucet-address"
              value={address}
              onChange={(event) => setAddress(event.target.value)}
              placeholder={t('faucetAddressPlaceholder')}
              autoComplete="off"
              spellCheck={false}
              className="font-mono text-xs"
              aria-invalid={showAddressHint}
            />
            <Button
              type="submit"
              disabled={!addressLooksValid || submit.kind === 'sending' || !status?.ready}
              className="shrink-0"
            >
              {submit.kind === 'sending' ? t('faucetSubmitting') : t('faucetSubmit')}
            </Button>
          </form>
          {showAddressHint && (
            <p className="text-xs text-red-700 dark:text-red-400">{t('faucetErrBadAddress')}</p>
          )}

          {submit.kind === 'sent' && (
            <Alert variant="success" className="space-y-2">
              <div className="flex items-center gap-2.5 font-semibold">
                <CheckCircle2 className="h-4 w-4 shrink-0" />
                <span>{t('faucetSuccessTitle')}</span>
              </div>
              <p className="text-xs leading-relaxed">
                {t('faucetSuccessBody', { amount: `${submit.payout.amount_zvrg} ZVRG` })}
              </p>
              <div className="flex flex-wrap items-center gap-2 text-xs">
                <span className="text-slate-600 dark:text-slate-300">{t('faucetTxLabel')}</span>
                <HashValue
                  value={submit.payout.txid}
                  href={routes.tx(submit.payout.txid)}
                  label="txid"
                />
              </div>
            </Alert>
          )}
          {submit.kind === 'failed' && (
            <Alert variant="destructive" className="flex items-start gap-3">
              <AlertTriangle className="h-4 w-4 shrink-0 mt-0.5" />
              <p className="text-xs leading-relaxed">{t(faucetErrorKey(submit.failure))}</p>
            </Alert>
          )}
        </CardContent>
      </Card>

      {/* How it works & privacy */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">{t('faucetHowTitle')}</CardTitle>
          </CardHeader>
          <CardContent>
            <ol className="list-decimal pl-5 space-y-1.5 text-xs leading-relaxed text-slate-700 dark:text-slate-300">
              <li>{t('faucetHowStep1')}</li>
              <li>{t('faucetHowStep2')}</li>
              <li>{t('faucetHowStep3')}</li>
            </ol>
          </CardContent>
        </Card>
        <Alert variant="info" className="flex items-start gap-3">
          <ShieldCheck className="h-4 w-4 shrink-0 mt-0.5" />
          <p className="text-xs leading-relaxed">{t('faucetPrivacyNote')}</p>
        </Alert>
      </div>
    </div>
  );
}
