'use client';

import React from 'react';
import Link from 'next/link';
import { Compass } from 'lucide-react';
import { useApp } from './providers';
import { Button, Card, CardContent } from '@/components/ui/primitives';

export default function NotFoundPage() {
  const { t } = useApp();

  return (
    <div className="py-16 flex justify-center">
      <Card className="max-w-md w-full text-center">
        <CardContent className="pt-8 pb-8 space-y-4">
          <div className="mx-auto flex h-12 w-12 items-center justify-center rounded-full bg-slate-100 dark:bg-slate-800 text-teal-600 dark:text-teal-400">
            <Compass className="h-6 w-6" />
          </div>
          <div className="space-y-1">
            <div className="font-mono text-xs text-slate-400">HTTP 404</div>
            <h1 className="text-lg font-bold text-slate-900 dark:text-slate-100">
              {t('notFoundTitle')}
            </h1>
            <p className="text-xs text-slate-500 dark:text-slate-400 leading-relaxed">
              {t('notFoundBody')}
            </p>
          </div>
          <div className="pt-2">
            <Button asChild>
              <Link href="/">{t('backToDashboard')}</Link>
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
