// Kerangka aplikasi: kepala halaman (navigasi, pencarian, bahasa), peringatan indexer, dan halaman sesuai jalur hash.
import { useEffect, useMemo, useState } from 'react';
import { browserApi } from './api';
import type { Status } from './api';
import { failureText } from './components/common';
import { STORAGE_KEY, detectLanguage, dict, fill } from './i18n';
import type { Lang } from './i18n';
import type { Load } from './load';
import { REFRESH_MS, useLoad } from './load';
import { AddressPage } from './pages/address';
import { AllocationPage } from './pages/allocation';
import { BlockPage } from './pages/block';
import { BlocksPage } from './pages/blocks';
import { HomePage } from './pages/home';
import { MempoolPage } from './pages/mempool';
import { SupplyPage } from './pages/supply';
import { TxPage } from './pages/tx';
import { href, isSearchable, parseRoute, searchHref } from './route';
import type { Route } from './route';
import type { Env } from './ui';

function storedLanguage(): string | null {
  try {
    return window.localStorage.getItem(STORAGE_KEY);
  } catch {
    return null;
  }
}

function storeLanguage(lang: Lang): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, lang);
  } catch {
    // Penyimpanan browser dimatikan: pilihan bahasa hanya berlaku sampai halaman ditutup.
  }
}

function Header({ env, onLanguage }: { env: Env; onLanguage: () => void }) {
  const { t, api } = env;
  const [query, setQuery] = useState('');
  const [message, setMessage] = useState<string | null>(null);
  return (
    <header className="header">
      <a className="brand" href={href({ name: 'home' })}>
        {t.appTitle}
      </a>
      <nav className="nav">
        <a href={href({ name: 'blocks', before: null })}>{t.navBlocks}</a>
        <a href={href({ name: 'supply' })}>{t.navSupply}</a>
        <a href={href({ name: 'allocation', page: '0' })}>{t.navAllocation}</a>
        <a href={href({ name: 'mempool' })}>{t.navMempool}</a>
      </nav>
      <form
        className="search"
        role="search"
        onSubmit={(event) => {
          event.preventDefault();
          const text = query.trim().toLowerCase();
          if (!isSearchable(text)) {
            setMessage(t.searchInvalid);
            return;
          }
          setMessage(null);
          void api.search(text).then((result) => {
            if (result.ok) {
              setQuery('');
              window.location.hash = searchHref(result.data);
            } else {
              setMessage(failureText(result.failure, t));
            }
          });
        }}
      >
        <input
          aria-label={t.searchLabel}
          placeholder={t.searchPlaceholder}
          value={query}
          spellCheck={false}
          autoComplete="off"
          onChange={(event) => setQuery(event.target.value)}
        />
        <button type="submit" className="button">
          {t.searchButton}
        </button>
      </form>
      <button type="button" className="button button-quiet" onClick={onLanguage}>
        {t.switchLanguage}
      </button>
      {message !== null && <p className="notice notice-error">{message}</p>}
    </header>
  );
}

function IndexWarning({ status, env }: { status: Load<Status>; env: Env }) {
  if (status.status !== 'ok' || status.data.index_error === null) {
    return null;
  }
  return (
    <div className="notice notice-error" role="alert">
      <strong>{env.t.indexErrorTitle}</strong>
      <p>{fill(env.t.indexErrorBody, { error: status.data.index_error })}</p>
    </div>
  );
}

function Page({ route, env, status }: { route: Route; env: Env; status: Load<Status> }) {
  switch (route.name) {
    case 'home':
      return <HomePage env={env} status={status} />;
    case 'blocks':
      return <BlocksPage env={env} before={route.before} />;
    case 'block':
      return <BlockPage env={env} blockKey={route.key} />;
    case 'tx':
      return <TxPage env={env} txid={route.txid} />;
    case 'address':
      return <AddressPage env={env} address={route.address} page={route.page} />;
    case 'supply':
      return <SupplyPage env={env} />;
    case 'allocation':
      return <AllocationPage env={env} page={route.page} />;
    case 'mempool':
      return <MempoolPage env={env} />;
    case 'notfound':
      return <p className="notice">{env.t.pageNotFound}</p>;
  }
}

export function App() {
  const api = useMemo(() => browserApi(), []);
  const [hash, setHash] = useState(() => window.location.hash);
  const [lang, setLang] = useState<Lang>(() => detectLanguage(storedLanguage(), window.navigator.language));
  useEffect(() => {
    const onHash = () => setHash(window.location.hash);
    window.addEventListener('hashchange', onHash);
    return () => window.removeEventListener('hashchange', onHash);
  }, []);
  const t = dict(lang);
  useEffect(() => {
    document.documentElement.lang = lang;
    document.title = t.appTitle;
  }, [lang, t]);
  const status = useLoad(() => api.status(), 'status', REFRESH_MS);
  const env: Env = { api, lang, t };
  const route = parseRoute(hash);
  return (
    <div className="app">
      <Header
        env={env}
        onLanguage={() => {
          const next: Lang = lang === 'id' ? 'en' : 'id';
          storeLanguage(next);
          setLang(next);
        }}
      />
      <main className="main">
        <IndexWarning status={status.load} env={env} />
        <Page key={hash} route={route} env={env} status={status.load} />
      </main>
      <footer className="footer">{t.privacy}</footer>
    </div>
  );
}
