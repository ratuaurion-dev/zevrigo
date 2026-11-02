'use client';

import React, { Suspense, useEffect, useState } from 'react';
import Link from 'next/link';
import { useRouter, useSearchParams } from 'next/navigation';
import { AlertTriangle, Search } from 'lucide-react';
import { useApp } from '@/app/providers';
import { fetchSearch, type ApiFailure } from '@/lib/api';
import { classifySearchQuery, routes } from '@/lib/route';
import { Alert, Button, Card, CardContent } from '@/components/ui/primitives';
import { ApiStateView, TableSkeleton } from '@/components/explorer/shared';

function SearchHandlerContent() {
  const { t } = useApp();
  const router = useRouter();
  const searchParams = useSearchParams();
  const rawQuery = searchParams.get('q') ?? '';
  const classified = classifySearchQuery(rawQuery);

  const [searching, setSearching] = useState(true);
  const [notFound, setNotFound] = useState(false);
  const [failure, setFailure] = useState<ApiFailure | null>(null);

  useEffect(() => {
    if (classified.kind === 'invalid') {
      setSearching(false);
      setNotFound(false);
      setFailure(null);
      return;
    }

    let cancelled = false;
    setSearching(true);
    setNotFound(false);
    setFailure(null);

    fetchSearch(classified.normalized).then((res) => {
      if (cancelled) return;
      if (res.ok) {
        const match = res.data;
        if (match.kind === 'block') {
          router.replace(routes.block(match.key));
        } else if (match.kind === 'tx') {
          router.replace(routes.tx(match.key));
        } else if (match.kind === 'address') {
          router.replace(routes.address(match.key));
        }
        return;
      }

      if (res.failure.kind === 'http' && res.failure.status === 404) {
        if (res.failure.inMempool === true && classified.kind === 'hash') {
          // Route to /tx/?id=<q> which renders the friendly mempool pending state
          router.replace(routes.tx(classified.normalized));
          return;
        }
        setSearching(false);
        setNotFound(true);
        return;
      }

      setSearching(false);
      setFailure(res.failure);
    });

    return () => {
      cancelled = true;
    };
  }, [classified.kind, classified.normalized, router]);

  if (classified.kind === 'invalid') {
    return (
      <div className="space-y-4">
        <Alert variant="warning">
          <div className="flex items-start gap-3">
            <AlertTriangle className="h-4 w-4 shrink-0 mt-0.5" />
            <div>
              <div className="font-semibold">{t('invalidInputTitle')}</div>
              <p className="mt-0.5">{t('invalidInputBody')}</p>
            </div>
          </div>
        </Alert>
        <Button variant="outline" size="sm" asChild>
          <Link href="/">
            {t('backToDashboard')}
          </Link>
        </Button>
      </div>
    );
  }

  if (searching) {
    return (
      <div className="py-12 text-center text-xs text-slate-500">
        {t('searchRedirecting')}
      </div>
    );
  }

  if (failure) {
    return <ApiStateView failure={failure} />;
  }

  if (notFound) {
    return (
      <div className="py-12 flex justify-center">
        <Card className="max-w-lg w-full">
          <CardContent className="p-6 space-y-4 text-center">
            <div className="mx-auto flex h-10 w-10 items-center justify-center rounded-full bg-slate-100 dark:bg-slate-800 text-slate-500">
              <Search className="h-5 w-5" />
            </div>
            <div className="space-y-1">
              <h1 className="text-base font-bold text-slate-900 dark:text-slate-100">
                {t('searchNothingFoundTitle', { query: classified.normalized })}
              </h1>
              <p className="text-xs text-slate-500 dark:text-slate-400 leading-relaxed">
                {t('searchNothingFoundBody')}
              </p>
            </div>
            <div>
              <Button variant="outline" size="sm" asChild>
                <Link href="/">
                  {t('backToDashboard')}
                </Link>
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  return null;
}

export default function SearchPage() {
  return (
    <Suspense fallback={<TableSkeleton rows={4} />}>
      <SearchHandlerContent />
    </Suspense>
  );
}
