/**
 * Pengaturan: alamat node per jaringan, bahasa, tema, lokasi file wallet, dan lihat frasa pemulihan (password).
 */

import { Eye, EyeOff, FolderOpen, Globe, KeyRound, Server } from 'lucide-react';
import { type FormEvent, useState } from 'react';
import { TEXT, walletErrorText } from '../lib/i18n';
import { ipc, toWalletError } from '../lib/ipc';
import { useWallet } from '../store';
import { Card, ErrorBox, PageHeader, inputClass, primaryButton, secondaryButton } from './ui';

function Choice<T extends string>({
  value,
  options,
  onChange,
}: {
  value: T;
  options: { key: T; label: string }[];
  onChange: (value: T) => void;
}) {
  return (
    <div className="grid grid-cols-2 gap-2">
      {options.map((o) => (
        <button
          key={o.key}
          type="button"
          onClick={() => onChange(o.key)}
          className={`py-2.5 px-3 text-xs font-semibold rounded-lg border transition-colors ${
            value === o.key
              ? 'bg-emerald-600 text-white border-emerald-600'
              : 'bg-slate-50 dark:bg-slate-950 border-slate-300 dark:border-slate-700 text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800'
          }`}
        >
          {o.label}
        </button>
      ))}
    </div>
  );
}

export function Settings() {
  const { language, theme, network, info, rpcUrl, rpcUrls, setRpcUrl, setLanguage, setTheme } = useWallet();
  const t = TEXT[language];
  const [rpcInput, setRpcInput] = useState(rpcUrl());
  const [password, setPassword] = useState('');
  const [phrase, setPhrase] = useState<string | null>(null);
  const [revealError, setRevealError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const reveal = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setBusy(true);
    setRevealError(null);
    try {
      setPhrase(await ipc.revealPhrase(network, password));
      setPassword('');
    } catch (err) {
      setPhrase(null);
      setRevealError(walletErrorText(t, toWalletError(err).code));
    } finally {
      setBusy(false);
    }
  };

  const saveRpc = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    await setRpcUrl(rpcInput);
  };

  const resetRpc = async () => {
    setRpcInput(info?.defaultRpcUrl ?? '');
    await setRpcUrl(null);
  };

  return (
    <div className="space-y-6">
      <PageHeader title={t.settingsTitle} />
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card className="space-y-4">
          <div className="flex items-center gap-2">
            <Server className="w-4 h-4 text-emerald-500" />
            <h2 className="text-sm font-semibold text-slate-900 dark:text-slate-100">
              {t.nodeTitle} · {network === 'testnet' ? t.networkTestnet : t.networkRegtest}
            </h2>
          </div>
          <p className="text-xs text-slate-500 dark:text-slate-400">
            {t.nodeDesc} <span className="font-mono">{info?.defaultRpcUrl}</span>
          </p>
          <form onSubmit={saveRpc} className="flex gap-2">
            <input
              type="text"
              spellCheck={false}
              value={rpcInput}
              onChange={(e) => setRpcInput(e.target.value)}
              className={`${inputClass} font-mono`}
            />
            <button type="submit" className={primaryButton}>
              {t.nodeSave}
            </button>
          </form>
          {rpcUrls[network] !== undefined && (
            <button type="button" onClick={resetRpc} className={secondaryButton}>
              {t.nodeReset}
            </button>
          )}
        </Card>

        <Card className="space-y-5">
          <div className="flex items-center gap-2">
            <Globe className="w-4 h-4 text-emerald-500" />
            <h2 className="text-sm font-semibold text-slate-900 dark:text-slate-100">{t.prefTitle}</h2>
          </div>
          <div className="space-y-2">
            <span className="block text-xs font-semibold text-slate-700 dark:text-slate-300">{t.language}</span>
            <Choice
              value={language}
              onChange={setLanguage}
              options={[
                { key: 'id', label: 'Bahasa Indonesia' },
                { key: 'en', label: 'English' },
              ]}
            />
          </div>
          <div className="space-y-2">
            <span className="block text-xs font-semibold text-slate-700 dark:text-slate-300">{t.theme}</span>
            <Choice
              value={theme}
              onChange={setTheme}
              options={[
                { key: 'dark', label: t.themeDark },
                { key: 'light', label: t.themeLight },
              ]}
            />
          </div>
        </Card>

        <Card className="space-y-3">
          <div className="flex items-center gap-2">
            <FolderOpen className="w-4 h-4 text-emerald-500" />
            <h2 className="text-sm font-semibold text-slate-900 dark:text-slate-100">{t.walletFileTitle}</h2>
          </div>
          <div className="font-mono text-xs text-slate-700 dark:text-slate-300 break-all">{info?.path}</div>
          <p className="text-xs text-slate-500 dark:text-slate-400">{t.walletFileDesc}</p>
        </Card>

        <Card className="space-y-4">
          <div className="flex items-center gap-2">
            <KeyRound className="w-4 h-4 text-emerald-500" />
            <h2 className="text-sm font-semibold text-slate-900 dark:text-slate-100">{t.revealTitle}</h2>
          </div>
          <p className="text-xs text-slate-500 dark:text-slate-400">{t.revealDesc}</p>
          {phrase === null ? (
            <form onSubmit={reveal} className="flex gap-2">
              <input
                type="password"
                autoComplete="current-password"
                aria-label={t.password}
                placeholder={t.password}
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className={inputClass}
                required
              />
              <button type="submit" disabled={busy || password === ''} className={primaryButton}>
                <Eye className="w-4 h-4" />
                {t.revealButton}
              </button>
            </form>
          ) : (
            <div className="space-y-3">
              <div
                data-testid="revealed-phrase"
                className="p-3.5 rounded-lg bg-slate-50 dark:bg-slate-950 border border-emerald-500/40 font-mono text-sm text-slate-900 dark:text-slate-100 leading-relaxed select-none"
              >
                {phrase}
              </div>
              <button type="button" onClick={() => setPhrase(null)} className={secondaryButton}>
                <EyeOff className="w-3.5 h-3.5" />
                {t.hideButton}
              </button>
            </div>
          )}
          {revealError && <ErrorBox>{revealError}</ErrorBox>}
        </Card>
      </div>
    </div>
  );
}
