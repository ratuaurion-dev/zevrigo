'use client';

import React, { useCallback } from 'react';
import { AlertOctagon, CheckCircle2, ShieldCheck } from 'lucide-react';
import { useApp } from '@/app/providers';
import { usePoll } from '@/hooks/use-poll';
import { fetchStatus, fetchSupply, type ApiResult, type Status, type SupplyInfo } from '@/lib/api';
import {
  computeBasisPoints,
  formatBasisPointsPercent,
  formatHsis,
  formatInt,
  MINING_SUPPLY_QUANTA,
} from '@/lib/amount';
import {
  Alert,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Progress,
} from '@/components/ui/primitives';
import { ApiStateView, HsisAmount, TableSkeleton } from '@/components/explorer/shared';

interface SupplyPageBundle {
  supply: SupplyInfo;
  status: Status;
}

export default function SupplyPage() {
  const { lang, t } = useApp();

  const fetcher = useCallback(async (): Promise<ApiResult<SupplyPageBundle>> => {
    const [supplyRes, statusRes] = await Promise.all([
      fetchSupply(),
      fetchStatus(),
    ]);
    if (!supplyRes.ok) return supplyRes;
    if (!statusRes.ok) return statusRes;
    return {
      ok: true,
      data: {
        supply: supplyRes.data,
        status: statusRes.data,
      },
    };
  }, []);

  const { data, failure, isLoading, reload } = usePoll(fetcher, []);

  if (failure && !data) {
    return <ApiStateView failure={failure} onRetry={reload} />;
  }

  if (isLoading && !data) {
    return <TableSkeleton rows={10} />;
  }

  const supply = data?.supply;
  const status = data?.status;
  if (!supply) return null;

  const minedBps = computeBasisPoints(supply.mined_total, MINING_SUPPLY_QUANTA);
  const minedPct = formatBasisPointsPercent(
    supply.mined_total,
    MINING_SUPPLY_QUANTA,
    lang
  );

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="border-b border-slate-200 dark:border-slate-800 pb-4 space-y-1">
        <h1 className="text-xl sm:text-2xl font-bold text-slate-900 dark:text-slate-100">
          {t('supplyPageTitle')}
        </h1>
        <p className="text-xs text-slate-500 dark:text-slate-400">
          {t('supplyPageSubtitle')}
        </p>
      </div>

      {/* Verification Status Banner */}
      {supply.matches_schedule ? (
        <Alert variant="success" className="flex items-center justify-between gap-4">
          <div className="flex items-center gap-2.5">
            <CheckCircle2 className="h-4 w-4 shrink-0 text-emerald-600 dark:text-emerald-400" />
            <span className="font-semibold">{t('supplyVerificationValid')}</span>
          </div>
          <span className="font-mono text-xs tabular-nums">
            #{formatInt(supply.height, lang)}
          </span>
        </Alert>
      ) : (
        <Alert variant="destructive">
          <div className="flex items-center gap-2.5 font-semibold">
            <AlertOctagon className="h-4 w-4 shrink-0" />
            <span>{t('supplyVerificationInvalid')}</span>
          </div>
        </Alert>
      )}

      {/* Key Supply Figures */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <Card>
          <CardContent className="p-4 space-y-1">
            <div className="text-xs text-slate-500 dark:text-slate-400">
              {t('supplyActualMined')}
            </div>
            <div className="text-base font-bold text-slate-900 dark:text-slate-100">
              <HsisAmount quanta={supply.mined_total} />
            </div>
            <div className="text-[11px] font-mono text-slate-500">
              {minedPct} / 43,800,000 HSIS
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-4 space-y-1">
            <div className="text-xs text-slate-500 dark:text-slate-400">
              {t('supplyScheduledMined')}
            </div>
            <div className="text-base font-bold text-slate-900 dark:text-slate-100">
              <HsisAmount quanta={supply.scheduled_mined_total} />
            </div>
            <div className="text-[11px] text-emerald-700 dark:text-emerald-400 font-medium">
              {supply.matches_schedule ? '✓ Exact match' : '✕ Schedule mismatch'}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-4 space-y-1">
            <div className="text-xs text-slate-500 dark:text-slate-400">
              {t('supplyGenesisAllocation')}
            </div>
            <div className="text-base font-bold text-slate-900 dark:text-slate-100">
              <HsisAmount quanta={supply.genesis_allocation} />
            </div>
            <div className="text-[11px] text-slate-500">
              Genesis Block #0 (outside S)
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-4 space-y-1">
            <div className="text-xs text-slate-500 dark:text-slate-400">
              {t('statTotalSupply')}
            </div>
            <div className="text-base font-bold text-teal-700 dark:text-teal-400">
              <HsisAmount quanta={supply.total_supply} />
            </div>
            <div className="text-[11px] text-slate-500">
              Mined + Genesis Allocation
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-4 space-y-1">
            <div className="text-xs text-slate-500 dark:text-slate-400">
              {t('supplyCumulativeFees')}
            </div>
            <div className="text-base font-bold text-slate-900 dark:text-slate-100">
              <HsisAmount quanta={supply.fees_total} />
            </div>
            <div className="text-[11px] text-slate-500">
              100% paid to miners (0 burned)
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-4 space-y-1">
            <div className="text-xs text-slate-500 dark:text-slate-400">
              {t('statNextSubsidy')}
            </div>
            <div className="text-base font-bold text-slate-900 dark:text-slate-100">
              <HsisAmount quanta={supply.next_subsidy} />
            </div>
            <div className="text-[11px] font-mono text-slate-500">
              Target 0x{supply.tip_target} · Avg{' '}
              {supply.average_block_seconds !== null
                ? `${formatInt(Math.round(supply.average_block_seconds), lang)}s`
                : '—'}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Mining Progress Bar */}
      <Card>
        <CardContent className="p-5 space-y-3">
          <div className="flex items-center justify-between text-xs font-medium">
            <span>{t('supplyProgressTitle')}</span>
            <span className="font-mono tabular-nums">{minedPct}</span>
          </div>
          <Progress
            basisPoints={minedBps}
            nonZero={supply.mined_total !== '0'}
            label={t('supplyProgressTitle')}
          />
          <div className="flex flex-wrap justify-between text-xs text-slate-500 font-mono tabular-nums">
            <span>
              M = <HsisAmount quanta={supply.mined_total} />
            </span>
            <span>S = {formatHsis(MINING_SUPPLY_QUANTA, lang)}</span>
          </div>
        </CardContent>
      </Card>

      {/* Emission Rules Explainer Card (Bilingual Static Protocol Specification) */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <ShieldCheck className="h-4 w-4 text-teal-600 dark:text-teal-400" />
            <CardTitle>{t('emissionRulesTitle')}</CardTitle>
          </div>
          <CardDescription className="font-semibold text-teal-800 dark:text-teal-300 pt-1">
            {t('emissionRulesAuditStatement')}
          </CardDescription>
        </CardHeader>
        <CardContent className="p-5 space-y-4 text-xs leading-relaxed">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="rounded-lg border border-slate-200 dark:border-slate-800 p-4 space-y-1">
              <h3 className="font-semibold text-slate-900 dark:text-slate-100">
                01. {t('emissionRuleUnitTitle')}
              </h3>
              <p className="text-slate-600 dark:text-slate-300">
                {t('emissionRuleUnitBody')}
              </p>
            </div>

            <div className="rounded-lg border border-slate-200 dark:border-slate-800 p-4 space-y-1">
              <h3 className="font-semibold text-slate-900 dark:text-slate-100">
                02. {t('emissionRuleCapTitle')}
              </h3>
              <p className="text-slate-600 dark:text-slate-300">
                {t('emissionRuleCapBody')}
              </p>
            </div>

            <div className="rounded-lg border border-slate-200 dark:border-slate-800 p-4 space-y-1">
              <h3 className="font-semibold text-slate-900 dark:text-slate-100">
                03. {t('emissionRuleFormulaTitle')}
              </h3>
              <p className="text-slate-600 dark:text-slate-300">
                {t('emissionRuleFormulaBody')}
              </p>
            </div>

            <div className="rounded-lg border border-slate-200 dark:border-slate-800 p-4 space-y-1">
              <h3 className="font-semibold text-slate-900 dark:text-slate-100">
                04. {t('emissionRuleSlowStartTitle')}
              </h3>
              <p className="text-slate-600 dark:text-slate-300">
                {t('emissionRuleSlowStartBody')}
              </p>
            </div>
          </div>

          <div className="rounded-lg border border-slate-200 dark:border-slate-800 p-4 space-y-1">
            <h3 className="font-semibold text-slate-900 dark:text-slate-100">
              05. {t('emissionRuleDifficultyTitle')}
            </h3>
            <p className="text-slate-600 dark:text-slate-300">
              {t('emissionRuleDifficultyBody')}
            </p>
          </div>

          {status?.chainwork && (
            <div className="pt-2 text-[11px] font-mono text-slate-500 dark:text-slate-400">
              {t('supplyChainwork')}: 0x{status.chainwork}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
