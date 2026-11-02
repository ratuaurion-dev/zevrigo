import React from 'react';
import type { Metadata } from 'next';
import './globals.css';
import { AppProviders } from './providers';
import { ExplorerShell } from '@/components/explorer/header';

export const metadata: Metadata = {
  title: 'Homeostasis Coin (HSIS) Block Explorer',
  description:
    'Block explorer Homeostasis Coin (HSIS): blok, transaksi, alamat, pasokan dibanding jadwal, alokasi pencipta, dan mempool.',
  referrer: 'no-referrer',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="id" className="dark" suppressHydrationWarning>
      <body className="min-h-screen bg-slate-50/60 dark:bg-slate-950 text-slate-900 dark:text-slate-100 antialiased selection:bg-teal-500/20 selection:text-teal-700 dark:selection:text-teal-300">
        <AppProviders>
          <ExplorerShell>{children}</ExplorerShell>
        </AppProviders>
      </body>
    </html>
  );
}
