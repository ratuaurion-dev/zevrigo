'use client';

import React, { useEffect, useState } from 'react';
import Link from 'next/link';
import { usePathname, useRouter } from 'next/navigation';
import {
  AlertOctagon,
  AlertTriangle,
  ArrowRight,
  Blocks,
  Coins,
  Cpu,
  Globe,
  LayoutDashboard,
  Menu,
  Moon,
  RefreshCw,
  Search,
  ShieldCheck,
  Sun,
  X,
} from 'lucide-react';
import { useApp } from '@/app/providers';
import { usePoll } from '@/hooks/use-poll';
import { fetchStatus } from '@/lib/api';
import { formatInt } from '@/lib/amount';
import { USE_MOCK } from '@/lib/config';
import { getSampleShowcaseIds } from '@/lib/mock';
import { routes, truncateMiddle } from '@/lib/route';
import { Alert, Button, Input } from '@/components/ui/primitives';
import { Dialog, DialogContent, DialogTitle } from '@/components/ui/dialog';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Sheet, SheetClose, SheetContent, SheetTitle } from '@/components/ui/sheet';
import { ZevrigoLogo } from './shared';
import { cn } from '@/lib/utils';

export function ExplorerShell({ children }: { children: React.ReactNode }) {
  const { lang, setLang, t, theme, toggleTheme } = useApp();
  const pathname = usePathname();
  const router = useRouter();

  const [cmdOpen, setCmdOpen] = useState(false);
  const [mobileOpen, setMobileOpen] = useState(false);
  const [searchInput, setSearchInput] = useState('');

  // Poll /api/v1/status every 10s on every page
  const { data: status } = usePoll(() => fetchStatus(), []);

  // Global keyboard shortcut Ctrl+K / ⌘K
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        setCmdOpen((prev) => !prev);
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, []);

  const handleSearchSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const q = searchInput.trim();
    if (!q) return;
    setCmdOpen(false);
    setMobileOpen(false);
    router.push(routes.search(q));
  };

  const navItems = [
    { href: '/', label: t('navDashboard'), match: (p: string) => p === '/' },
    {
      href: '/blocks/',
      label: t('navBlocks'),
      match: (p: string) => p.startsWith('/blocks') || p.startsWith('/block'),
    },
    {
      href: '/supply/',
      label: t('navSupply'),
      match: (p: string) => p.startsWith('/supply'),
    },
    {
      href: '/mempool/',
      label: t('navMempool'),
      match: (p: string) => p.startsWith('/mempool'),
    },
  ];

  const networkName = (status?.network ?? 'regtest').toUpperCase();
  const showcase = getSampleShowcaseIds();

  return (
    <div className="min-h-screen flex flex-col bg-slate-50/60 dark:bg-slate-950 text-slate-900 dark:text-slate-100">
      {/* Sticky Top Header */}
      <header className="sticky top-0 z-40 border-b border-slate-200 dark:border-slate-800/90 bg-white/95 dark:bg-slate-950/95 backdrop-blur-xs">
        <div className="mx-auto flex max-w-7xl items-center justify-between gap-2 sm:gap-6 px-3 sm:px-6 h-14">
          {/* Zone 1: Brand lockup + network tag */}
          <div className="flex items-center gap-2 sm:gap-3 min-w-0 shrink-0">
            <Link
              href="/"
              className="flex items-center gap-2 text-sm font-bold tracking-tight text-slate-900 dark:text-slate-100 whitespace-nowrap shrink-0"
            >
              <ZevrigoLogo className="w-5 h-5 sm:w-6 sm:h-6 shrink-0" />
              <span>Zevrigo</span>
              <span className="hidden sm:inline font-mono text-xs font-medium text-teal-700 dark:text-teal-400">
                ZVRG
              </span>
            </Link>
            <span
              aria-hidden="true"
              className="hidden sm:inline text-slate-300 dark:text-slate-700"
            >
              /
            </span>
            <span
              className={cn(
                'hidden sm:inline font-mono text-[11px] font-semibold tracking-wider uppercase',
                networkName === 'MAINNET'
                  ? 'text-emerald-700 dark:text-emerald-400'
                  : networkName === 'TESTNET'
                  ? 'text-sky-700 dark:text-sky-400'
                  : 'text-amber-700 dark:text-amber-400'
              )}
            >
              {networkName}
            </span>
          </div>

          {/* Zone 2: Primary Navigation Links */}
          <nav
            aria-label="Main Navigation"
            className="hidden lg:flex items-center gap-6 text-xs font-medium"
          >
            {navItems.map((item) => {
              const active = item.match(pathname || '/');
              return (
                <Link
                  key={item.href}
                  href={item.href}
                  className={cn(
                    'py-1 transition-colors whitespace-nowrap shrink-0 border-b-2',
                    active
                      ? 'border-teal-600 dark:border-teal-400 text-slate-900 dark:text-slate-100 font-semibold'
                      : 'border-transparent text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200'
                  )}
                >
                  {item.label}
                </Link>
              );
            })}
          </nav>

          {/* Zone 3: Search bar trigger, Language switch, Theme toggle */}
          <div className="flex items-center gap-1 sm:gap-2 shrink-0">
            <button
              type="button"
              onClick={() => setCmdOpen(true)}
              aria-label={t('searchTriggerLabel')}
              className="hidden sm:inline-flex items-center justify-between gap-3 h-8 w-56 md:w-64 rounded-md border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-900 px-3 text-xs text-slate-500 dark:text-slate-400 hover:border-slate-300 dark:hover:border-slate-700 transition-colors cursor-pointer"
            >
              <span className="inline-flex items-center gap-2 truncate">
                <Search className="h-3.5 w-3.5 shrink-0" />
                <span className="truncate">{t('searchTriggerLabel')}</span>
              </span>
              <kbd className="rounded border border-slate-200 dark:border-slate-700 bg-white dark:bg-slate-800 px-1.5 py-0.5 font-mono text-[10px] text-slate-500 dark:text-slate-400 shrink-0">
                ⌘K
              </kbd>
            </button>

            <Button
              variant="outline"
              size="icon"
              className="sm:hidden h-8 w-8"
              onClick={() => setCmdOpen(true)}
              aria-label={t('searchButton')}
            >
              <Search className="h-4 w-4" />
            </Button>

            {/* Language Switcher (ID / EN) */}
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button
                  variant="outline"
                  size="sm"
                  aria-label={t('langSwitchLabel')}
                  className="h-8 px-2 sm:px-2.5 font-mono uppercase"
                >
                  <Globe className="h-3.5 w-3.5" />
                  <span>{lang.toUpperCase()}</span>
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuItem
                  onSelect={() => setLang('id')}
                  className={cn(
                    lang === 'id' &&
                      'bg-teal-50 dark:bg-teal-950/60 text-teal-800 dark:text-teal-300 font-semibold'
                  )}
                >
                  <span>Bahasa Indonesia</span>
                  <span className="font-mono text-[10px]">ID</span>
                </DropdownMenuItem>
                <DropdownMenuItem
                  onSelect={() => setLang('en')}
                  className={cn(
                    lang === 'en' &&
                      'bg-teal-50 dark:bg-teal-950/60 text-teal-800 dark:text-teal-300 font-semibold'
                  )}
                >
                  <span>English</span>
                  <span className="font-mono text-[10px]">EN</span>
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>

            {/* Theme Toggle */}
            <Button
              variant="outline"
              size="icon"
              className="h-8 w-8"
              onClick={toggleTheme}
              aria-label={theme === 'dark' ? t('themeToggleLight') : t('themeToggleDark')}
              title={theme === 'dark' ? t('themeToggleLight') : t('themeToggleDark')}
            >
              {theme === 'dark' ? (
                <Sun className="h-4 w-4 text-amber-400" />
              ) : (
                <Moon className="h-4 w-4 text-slate-700" />
              )}
            </Button>

            {/* Mobile Hamburger Button */}
            <Button
              variant="outline"
              size="icon"
              className="lg:hidden h-8 w-8"
              onClick={() => setMobileOpen(true)}
              aria-label={t('mobileMenuTitle')}
            >
              <Menu className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </header>

      {/* Global Status Banners from /api/v1/status */}
      {status && (
        <div className="mx-auto w-full max-w-7xl px-4 sm:px-6">
          {status.index_error && (
            <Alert variant="destructive" className="mt-4">
              <div className="flex items-start gap-3">
                <AlertOctagon className="h-4 w-4 shrink-0 mt-0.5" />
                <div className="space-y-1">
                  <div className="font-semibold">{t('bannerIndexErrorTitle')}</div>
                  <div className="text-[11px] opacity-90">{t('bannerIndexErrorBody')}</div>
                </div>
              </div>
            </Alert>
          )}

          {status.last_error && (
            <Alert variant="warning" className="mt-4">
              <div className="flex items-start gap-3">
                <AlertTriangle className="h-4 w-4 shrink-0 mt-0.5" />
                <div className="space-y-1">
                  <div className="font-semibold">{t('bannerLastErrorTitle')}</div>
                  <div className="text-[11px] opacity-90">{t('bannerLastErrorBody')}</div>
                </div>
              </div>
            </Alert>
          )}

          {status.indexed_height !== null &&
            status.node_height !== null &&
            status.indexed_height < status.node_height && (
              <Alert variant="info" className="mt-4">
                <div className="flex items-center gap-2.5">
                  <RefreshCw className="h-3.5 w-3.5 animate-spin shrink-0" />
                  <span className="font-medium">
                    {t('bannerSyncingText', {
                      indexed: formatInt(status.indexed_height, lang),
                      node: formatInt(status.node_height, lang),
                    })}
                  </span>
                </div>
              </Alert>
            )}
        </div>
      )}

      {/* Main Content Viewport */}
      <main className="mx-auto w-full max-w-7xl flex-1 px-4 sm:px-6 py-6">
        {children}
      </main>

      {/* Footer */}
      <footer className="border-t border-slate-200 dark:border-slate-800/80 bg-white dark:bg-slate-950 py-6 mt-12">
        <div className="mx-auto max-w-7xl px-4 sm:px-6 flex flex-col gap-4">
          <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 text-xs text-slate-500 dark:text-slate-400">
            <p className="max-w-3xl leading-relaxed">{t('footerPrivacyNote')}</p>
            <div className="flex items-center gap-4 shrink-0">
              <span>{t('footerProtocolNote')}</span>
            </div>
          </div>

          {/* In Mock Mode, allow one-click preview of error/syncing states */}
          {USE_MOCK && (
            <div className="flex flex-wrap items-center gap-2 pt-3 border-t border-slate-100 dark:border-slate-900 text-[11px] text-slate-500 dark:text-slate-400">
              <span className="font-medium">{t('mockPreviewLabel')}</span>
              <a
                href={pathname || '/'}
                className="hover:text-teal-600 dark:hover:text-teal-400 underline"
              >
                {t('mockStateNormal')}
              </a>
              <span aria-hidden="true">·</span>
              <a
                href={`${pathname || '/'}?mock=notready`}
                className="hover:text-teal-600 dark:hover:text-teal-400 underline"
              >
                {t('mockStateNotReady')}
              </a>
              <span aria-hidden="true">·</span>
              <a
                href={`${pathname || '/'}?mock=indexerror`}
                className="hover:text-teal-600 dark:hover:text-teal-400 underline"
              >
                {t('mockStateIndexError')}
              </a>
              <span aria-hidden="true">·</span>
              <a
                href={`${pathname || '/'}?mock=offline`}
                className="hover:text-teal-600 dark:hover:text-teal-400 underline"
              >
                {t('mockStateOffline')}
              </a>
            </div>
          )}
        </div>
      </footer>

      {/* Command Palette / Quick Search Modal (Ctrl+K / ⌘K) */}
      <Dialog open={cmdOpen} onOpenChange={setCmdOpen}>
        <DialogContent aria-describedby={undefined}>
        <DialogTitle className="sr-only">{t('searchCommandTitle')}</DialogTitle>
        <div className="p-4">
          <form onSubmit={handleSearchSubmit} className="flex items-center gap-2">
            <div className="relative flex-1">
              <Search className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-slate-400" />
              <Input
                autoFocus
                value={searchInput}
                onChange={(e) => setSearchInput(e.target.value)}
                placeholder={t('searchTriggerLabel')}
                aria-label={t('searchTriggerLabel')}
                className="pl-9 h-10 text-xs font-mono"
              />
            </div>
            <Button type="submit" className="h-10 px-4">
              {t('searchButton')}
            </Button>
          </form>

          <p className="mt-2.5 text-[11px] text-slate-500 dark:text-slate-400">
            {t('searchCommandEmpty')}
          </p>

          {/* Contoh data tiruan (hanya mode mock, NEXT_PUBLIC_USE_MOCK=1) */}
          {USE_MOCK && (
          <div className="mt-4 pt-3 border-t border-slate-100 dark:border-slate-800 space-y-3">
            <div>
              <div className="text-[11px] font-semibold text-slate-500 dark:text-slate-400 mb-1.5">
                {t('searchSampleEntities')}
              </div>
              <div className="grid grid-cols-1 gap-1">
                <button
                  type="button"
                  onClick={() => {
                    setCmdOpen(false);
                    router.push(routes.block(120));
                  }}
                  className="flex items-center justify-between rounded-md px-2.5 py-1.5 text-xs hover:bg-slate-100 dark:hover:bg-slate-800 text-left cursor-pointer"
                >
                  <span className="inline-flex items-center gap-2 font-medium">
                    <Blocks className="h-3.5 w-3.5 text-teal-600 dark:text-teal-400" />
                    <span>Block #120 (Chain Tip)</span>
                  </span>
                  <span className="font-mono text-[11px] text-slate-400">
                    {truncateMiddle(showcase.tipBlockId, 8, 6)}
                  </span>
                </button>

                <button
                  type="button"
                  onClick={() => {
                    setCmdOpen(false);
                    router.push(routes.block(0));
                  }}
                  className="flex items-center justify-between rounded-md px-2.5 py-1.5 text-xs hover:bg-slate-100 dark:hover:bg-slate-800 text-left cursor-pointer"
                >
                  <span className="inline-flex items-center gap-2 font-medium">
                    <ShieldCheck className="h-3.5 w-3.5 text-teal-600 dark:text-teal-400" />
                    <span>Block #0 (Genesis)</span>
                  </span>
                  <span className="font-mono text-[11px] text-slate-400">
                    {truncateMiddle(showcase.genesisBlockId, 8, 6)}
                  </span>
                </button>

                <button
                  type="button"
                  onClick={() => {
                    setCmdOpen(false);
                    router.push(routes.tx(showcase.spendTxId));
                  }}
                  className="flex items-center justify-between rounded-md px-2.5 py-1.5 text-xs hover:bg-slate-100 dark:hover:bg-slate-800 text-left cursor-pointer"
                >
                  <span className="inline-flex items-center gap-2 font-medium">
                    <Coins className="h-3.5 w-3.5 text-amber-600 dark:text-amber-400" />
                    <span>Tx @ Height #100 (Genesis Spend, Fee 1,000 quanta)</span>
                  </span>
                  <span className="font-mono text-[11px] text-slate-400">
                    {truncateMiddle(showcase.spendTxId, 8, 6)}
                  </span>
                </button>

                <button
                  type="button"
                  onClick={() => {
                    setCmdOpen(false);
                    router.push(routes.tx(showcase.mempoolPendingTxId));
                  }}
                  className="flex items-center justify-between rounded-md px-2.5 py-1.5 text-xs hover:bg-slate-100 dark:hover:bg-slate-800 text-left cursor-pointer"
                >
                  <span className="inline-flex items-center gap-2 font-medium">
                    <Cpu className="h-3.5 w-3.5 text-sky-600 dark:text-sky-400" />
                    <span>Pending Mempool Tx (404 + in_mempool: true)</span>
                  </span>
                  <span className="font-mono text-[11px] text-slate-400">
                    {truncateMiddle(showcase.mempoolPendingTxId, 8, 6)}
                  </span>
                </button>
              </div>
            </div>
          </div>
          )}
        </div>
        </DialogContent>
      </Dialog>

      {/* Mobile Navigation Sheet */}
      <Sheet open={mobileOpen} onOpenChange={setMobileOpen}>
        <SheetContent aria-describedby={undefined}>
        <div className="flex items-center justify-between pb-4 border-b border-slate-200 dark:border-slate-800">
          <SheetTitle className="flex items-center gap-2 font-bold text-sm">
            <ZevrigoLogo className="w-5 h-5" />
            <span>Zevrigo (ZVRG)</span>
          </SheetTitle>
          <SheetClose asChild>
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8"
              aria-label={t('mobileMenuClose')}
            >
              <X className="h-4 w-4" />
            </Button>
          </SheetClose>
        </div>
        <div
          className={cn(
            'mt-3 font-mono text-[11px] font-semibold tracking-wider uppercase',
            networkName === 'MAINNET'
              ? 'text-emerald-700 dark:text-emerald-400'
              : networkName === 'TESTNET'
              ? 'text-sky-700 dark:text-sky-400'
              : 'text-amber-700 dark:text-amber-400'
          )}
        >
          {networkName}
        </div>

        <nav className="mt-4 flex flex-col gap-1">
          {navItems.map((item) => {
            const active = item.match(pathname || '/');
            return (
              <Link
                key={item.href}
                href={item.href}
                onClick={() => setMobileOpen(false)}
                className={cn(
                  'flex items-center gap-2.5 rounded-md px-3 py-2 text-xs font-medium transition-colors',
                  active
                    ? 'bg-teal-50 dark:bg-teal-950/60 text-teal-800 dark:text-teal-300 font-semibold'
                    : 'text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800'
                )}
              >
                <LayoutDashboard className="h-3.5 w-3.5 opacity-70" />
                <span>{item.label}</span>
              </Link>
            );
          })}
        </nav>
        </SheetContent>
      </Sheet>
    </div>
  );
}
