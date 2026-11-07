/**
 * Kerangka wallet desktop: bilah atas (nama, navigasi), bilah status (node, tinggi blok, jaringan, bahasa, tema), dan
 * layar aktif. Tanpa wallet di jaringan aktif, hanya layar wallet baru / pulihkan yang tampil.
 */

import { Blocks, CircleCheck, Moon, OctagonAlert, RefreshCw, Sun } from 'lucide-react';
import { type ReactNode, useEffect } from 'react';
import { Dashboard } from './components/Dashboard';
import { History } from './components/History';
import { Onboarding } from './components/Onboarding';
import { Receive } from './components/Receive';
import { Send } from './components/Send';
import { Settings } from './components/Settings';
import { ErrorBox } from './components/ui';
import { NETWORKS } from './lib/address';
import { TEXT, walletErrorText } from './lib/i18n';
import { type Tab, useWallet } from './store';

const REFRESH_MS = 20_000;

export default function App() {
  const state = useWallet();
  const { language, theme, network, tab, info, infoError, connection, chain, setTab, setNetwork, setLanguage, setTheme } =
    state;
  const t = TEXT[language];

  useEffect(() => {
    void useWallet.getState().loadWallet();
  }, []);

  useEffect(() => {
    document.documentElement.classList.toggle('dark', theme === 'dark');
    document.documentElement.lang = language;
  }, [theme, language]);

  useEffect(() => {
    if (!info?.exists) {
      return undefined;
    }
    const timer = setInterval(() => void useWallet.getState().refresh(), REFRESH_MS);
    return () => clearInterval(timer);
  }, [info?.exists]);

  const nav: { key: Tab; label: string }[] = [
    { key: 'dashboard', label: t.navDashboard },
    { key: 'receive', label: t.navReceive },
    { key: 'send', label: t.navSend },
    { key: 'history', label: t.navHistory },
    { key: 'settings', label: t.navSettings },
  ];

  let body: ReactNode;
  if (infoError) {
    body = <ErrorBox>{walletErrorText(t, infoError.code)}</ErrorBox>;
  } else if (info === null) {
    body = <div className="text-sm text-slate-500 dark:text-slate-400">{t.loading}</div>;
  } else if (!info.exists) {
    body = <Onboarding />;
  } else {
    body = {
      dashboard: <Dashboard />,
      receive: <Receive />,
      send: <Send />,
      history: <History />,
      settings: <Settings />,
    }[tab];
  }

  return (
    <div className="min-h-screen flex flex-col bg-slate-50 dark:bg-slate-950 text-slate-900 dark:text-slate-100">
      <header className="flex items-center justify-between gap-8 px-6 py-4 bg-white dark:bg-slate-900 border-b border-slate-200 dark:border-slate-800">
        <div className="text-lg font-bold tracking-tight whitespace-nowrap">{t.appName}</div>
        {info?.exists && (
          <nav aria-label="Navigasi" className="flex items-center gap-6 text-sm font-medium text-slate-600 dark:text-slate-400 overflow-x-auto">
            {nav.map((item) => (
              <button
                key={item.key}
                type="button"
                onClick={() => setTab(item.key)}
                className={`py-1 border-b-2 whitespace-nowrap transition-colors ${
                  tab === item.key
                    ? 'border-emerald-500 text-slate-900 dark:text-slate-100 font-semibold'
                    : 'border-transparent hover:text-slate-900 dark:hover:text-slate-200'
                }`}
              >
                {item.label}
              </button>
            ))}
          </nav>
        )}
      </header>

      <div className="bg-slate-100/80 dark:bg-slate-900/60 border-b border-slate-200 dark:border-slate-800/80 px-6 py-2.5">
        <div className="max-w-6xl mx-auto flex flex-wrap items-center justify-between gap-4 text-xs">
          <div className="flex flex-wrap items-center gap-4">
            {info?.exists && (
              <span data-testid="connection" className="inline-flex items-center gap-1.5 font-medium">
                {connection === 'online' && (
                  <>
                    <CircleCheck className="w-3.5 h-3.5 text-emerald-500" />
                    <span className="text-emerald-700 dark:text-emerald-400">{t.statusOnline}</span>
                  </>
                )}
                {(connection === 'connecting' || connection === 'idle') && (
                  <>
                    <RefreshCw className="w-3.5 h-3.5 text-amber-500 animate-spin" />
                    <span className="text-amber-700 dark:text-amber-400">{t.statusConnecting}</span>
                  </>
                )}
                {connection === 'offline' && (
                  <>
                    <OctagonAlert className="w-3.5 h-3.5 text-rose-500" />
                    <span className="text-rose-700 dark:text-rose-400">{t.statusOffline}</span>
                  </>
                )}
                <span className="text-slate-400">·</span>
                <span className="font-mono text-slate-500 dark:text-slate-400">{state.rpcUrl()}</span>
              </span>
            )}
            {chain && (
              <span className="inline-flex items-center gap-1.5 font-mono tabular-nums text-slate-600 dark:text-slate-300">
                <Blocks className="w-3.5 h-3.5 text-slate-400" />
                {t.blockHeight}: <span className="font-semibold">{chain.height.toLocaleString(language === 'id' ? 'id-ID' : 'en-US')}</span>
              </span>
            )}
          </div>
          <div className="flex items-center gap-3">
            <div
              role="group"
              aria-label={t.networkLabel}
              className="inline-flex p-0.5 rounded-lg bg-white dark:bg-slate-950 border border-slate-300 dark:border-slate-800"
            >
              {NETWORKS.map((net) => (
                <button
                  key={net}
                  type="button"
                  onClick={() => void setNetwork(net)}
                  className={`px-2.5 py-1 text-[11px] font-semibold rounded-md transition-colors ${
                    network === net ? 'bg-emerald-600 text-white' : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200'
                  }`}
                >
                  {net === 'testnet' ? t.networkTestnet : t.networkRegtest}
                </button>
              ))}
            </div>
            <button
              type="button"
              aria-label={t.toggleLanguage}
              onClick={() => setLanguage(language === 'id' ? 'en' : 'id')}
              className="px-2.5 py-1 text-[11px] font-mono font-semibold rounded-lg bg-white dark:bg-slate-950 border border-slate-300 dark:border-slate-800"
            >
              {language.toUpperCase()}
            </button>
            <button
              type="button"
              aria-label={t.toggleTheme}
              onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
              className="p-1.5 rounded-lg bg-white dark:bg-slate-950 border border-slate-300 dark:border-slate-800"
            >
              {theme === 'dark' ? <Sun className="w-3.5 h-3.5 text-amber-400" /> : <Moon className="w-3.5 h-3.5" />}
            </button>
          </div>
        </div>
      </div>

      <main className="flex-1 w-full max-w-6xl mx-auto px-6 py-8">{body}</main>

      <footer className="border-t border-slate-200 dark:border-slate-800/80 px-6 py-4 text-xs text-slate-500 dark:text-slate-400">
        <div className="max-w-6xl mx-auto">{t.footer}</div>
      </footer>
    </div>
  );
}
