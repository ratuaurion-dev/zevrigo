import React from 'react';
import type { Metadata } from 'next';
import './globals.css';
import { DemoNotice, SiteFooter, SiteHeader } from '@/components/site';

export const metadata: Metadata = {
  metadataBase: new URL('https://zevrigo-blog.ratuaurion.store'),
  title: {
    default: 'Blog Zevrigo (ZVRG)',
    template: '%s · Blog Zevrigo',
  },
  description:
    'Blog resmi Zevrigo (ZVRG): kabar pengembangan, panduan jaringan demo, protokol Proof-of-Work, dan tokenomik.',
  referrer: 'no-referrer',
  openGraph: {
    siteName: 'Blog Zevrigo',
    locale: 'id_ID',
    type: 'website',
  },
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="id">
      <body className="min-h-screen bg-[#FAF9F6] dark:bg-[#09090B] text-[#18181B] dark:text-[#F4F4F5] antialiased selection:bg-[#0F766E]/15 selection:text-[#0F766E] dark:selection:bg-[#2DD4BF]/20 dark:selection:text-[#2DD4BF]">
        <SiteHeader />
        <DemoNotice />
        <main className="mx-auto max-w-6xl px-5 md:px-8">{children}</main>
        <SiteFooter />
      </body>
    </html>
  );
}
