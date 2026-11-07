/**
 * Wallet baru atau pemulihan. Frasa baru dibuat backend Rust (sumber acak sistem operasi) dan baru disimpan sesudah
 * pengguna mengisi ulang tiga kata dari catatannya. File wallet selalu dikunci password (Argon2id).
 */

import { ArrowLeft, ArrowRight, KeyRound, RefreshCw, ShieldCheck, TriangleAlert } from 'lucide-react';
import { type FormEvent, useMemo, useState } from 'react';
import { checkPhrase, quizPositions } from '../lib/bip39';
import { TEXT, walletErrorText } from '../lib/i18n';
import { ipc, toWalletError } from '../lib/ipc';
import { useWallet } from '../store';
import { Card, ErrorBox, PageHeader, inputClass, primaryButton, secondaryButton } from './ui';

function PasswordFields({
  password,
  confirm,
  onPassword,
  onConfirm,
}: {
  password: string;
  confirm: string;
  onPassword: (v: string) => void;
  onConfirm: (v: string) => void;
}) {
  const t = TEXT[useWallet((s) => s.language)];
  return (
    <div className="space-y-2">
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <label className="space-y-1.5 text-xs font-semibold text-slate-700 dark:text-slate-300">
          <span className="block">{t.password}</span>
          <input
            type="password"
            autoComplete="new-password"
            value={password}
            onChange={(e) => onPassword(e.target.value)}
            className={inputClass}
            required
          />
        </label>
        <label className="space-y-1.5 text-xs font-semibold text-slate-700 dark:text-slate-300">
          <span className="block">{t.passwordConfirm}</span>
          <input
            type="password"
            autoComplete="new-password"
            value={confirm}
            onChange={(e) => onConfirm(e.target.value)}
            className={inputClass}
            required
          />
        </label>
      </div>
      <p className="text-xs text-slate-500 dark:text-slate-400">{t.passwordHint}</p>
    </div>
  );
}

function CreateWallet() {
  const { language, network, walletReady } = useWallet();
  const t = TEXT[language];
  const [step, setStep] = useState<'intro' | 'show' | 'verify'>('intro');
  const [phrase, setPhrase] = useState('');
  const [written, setWritten] = useState(false);
  const [positions, setPositions] = useState<number[]>([]);
  const [answers, setAnswers] = useState<string[]>(['', '', '']);
  const [password, setPassword] = useState('');
  const [confirm, setConfirm] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const words = useMemo(() => (phrase ? phrase.split(' ') : []), [phrase]);

  const makePhrase = async () => {
    setBusy(true);
    setError(null);
    try {
      const next = await ipc.newPhrase();
      setPhrase(next);
      setPositions(quizPositions(next.split(' ').length));
      setAnswers(['', '', '']);
      setWritten(false);
      setStep('show');
    } catch (e) {
      setError(walletErrorText(t, toWalletError(e).code));
    } finally {
      setBusy(false);
    }
  };

  const submit = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setError(null);
    const matches = positions.every((p, i) => (answers[i] ?? '').trim().toLowerCase() === words[p - 1]);
    if (!matches) {
      setError(t.verifyMismatch);
      return;
    }
    setBusy(true);
    try {
      const address = await ipc.restoreWallet(network, phrase, password, confirm);
      setPhrase('');
      setPassword('');
      setConfirm('');
      await walletReady(address);
    } catch (err) {
      setError(walletErrorText(t, toWalletError(err).code));
    } finally {
      setBusy(false);
    }
  };

  if (step === 'intro') {
    return (
      <Card className="space-y-5">
        <div className="flex items-start gap-3">
          <ShieldCheck className="w-5 h-5 text-emerald-500 shrink-0 mt-0.5" />
          <p className="text-sm text-slate-700 dark:text-slate-300">{t.createIntro}</p>
        </div>
        {error && <ErrorBox>{error}</ErrorBox>}
        <div className="flex justify-end">
          <button type="button" onClick={makePhrase} disabled={busy} className={primaryButton}>
            <KeyRound className="w-4 h-4" />
            {t.createStart}
          </button>
        </div>
      </Card>
    );
  }

  if (step === 'show') {
    return (
      <Card className="space-y-5">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
          <h2 className="text-lg font-semibold text-slate-900 dark:text-slate-100">{t.showTitle}</h2>
          <button type="button" onClick={makePhrase} disabled={busy} className={secondaryButton}>
            <RefreshCw className="w-3.5 h-3.5" />
            {t.showRegenerate}
          </button>
        </div>
        <ol
          data-testid="phrase-grid"
          className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-2.5 p-4 rounded-lg bg-slate-50 dark:bg-slate-950 border border-slate-200 dark:border-slate-800 select-none"
        >
          {words.map((word, i) => (
            <li
              key={i}
              className="flex items-center gap-2 px-3 py-2 rounded-md bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 font-mono text-sm"
            >
              <span className="text-slate-400 tabular-nums w-6 text-right">{i + 1}.</span>
              <span className="font-semibold text-slate-900 dark:text-slate-100">{word}</span>
            </li>
          ))}
        </ol>
        <div className="flex items-start gap-3 p-4 rounded-lg bg-amber-500/10 border border-amber-500/30 text-amber-800 dark:text-amber-300 text-xs">
          <TriangleAlert className="w-4 h-4 shrink-0 mt-0.5" />
          <span>{t.showWarning}</span>
        </div>
        <label className="flex items-center gap-2 text-sm text-slate-700 dark:text-slate-300">
          <input type="checkbox" checked={written} onChange={(e) => setWritten(e.target.checked)} />
          {t.showConfirm}
        </label>
        {error && <ErrorBox>{error}</ErrorBox>}
        <div className="flex justify-between">
          <button type="button" onClick={() => setStep('intro')} className={secondaryButton}>
            <ArrowLeft className="w-3.5 h-3.5" />
            {t.back}
          </button>
          <button type="button" disabled={!written} onClick={() => setStep('verify')} className={primaryButton}>
            {t.next}
            <ArrowRight className="w-4 h-4" />
          </button>
        </div>
      </Card>
    );
  }

  return (
    <Card>
      <form onSubmit={submit} className="space-y-5">
        <div>
          <h2 className="text-lg font-semibold text-slate-900 dark:text-slate-100">{t.verifyTitle}</h2>
          <p className="text-sm text-slate-600 dark:text-slate-400 mt-0.5">{t.verifyDesc}</p>
        </div>
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
          {positions.map((position, i) => (
            <label key={position} className="space-y-1.5 text-xs font-semibold text-slate-700 dark:text-slate-300">
              <span className="block">
                {t.wordNumber}
                <span className="font-mono tabular-nums">{position}</span>
              </span>
              <input
                type="text"
                autoComplete="off"
                spellCheck={false}
                value={answers[i] ?? ''}
                onChange={(e) => setAnswers(answers.map((a, j) => (j === i ? e.target.value : a)))}
                className={`${inputClass} font-mono`}
                required
              />
            </label>
          ))}
        </div>
        <PasswordFields password={password} confirm={confirm} onPassword={setPassword} onConfirm={setConfirm} />
        {error && <ErrorBox>{error}</ErrorBox>}
        <div className="flex justify-between">
          <button type="button" onClick={() => setStep('show')} className={secondaryButton}>
            <ArrowLeft className="w-3.5 h-3.5" />
            {t.back}
          </button>
          <button type="submit" disabled={busy} className={primaryButton}>
            <ShieldCheck className="w-4 h-4" />
            {busy ? t.loading : t.createSubmit}
          </button>
        </div>
      </form>
    </Card>
  );
}

function RestoreWallet() {
  const { language, network, walletReady } = useWallet();
  const t = TEXT[language];
  const [input, setInput] = useState('');
  const [password, setPassword] = useState('');
  const [confirm, setConfirm] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const check = useMemo(() => checkPhrase(input), [input]);
  const ready = check.countOk && check.unknown.length === 0;

  const submit = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setError(null);
    if (!ready) {
      setError(check.countOk ? t.walletErrors.bad_phrase : t.restoreCountBad);
      return;
    }
    setBusy(true);
    try {
      const address = await ipc.restoreWallet(network, check.normalized, password, confirm);
      setInput('');
      setPassword('');
      setConfirm('');
      await walletReady(address);
    } catch (err) {
      setError(walletErrorText(t, toWalletError(err).code));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Card>
      <form onSubmit={submit} className="space-y-5">
        <div>
          <h2 className="text-lg font-semibold text-slate-900 dark:text-slate-100">{t.restoreTitle}</h2>
          <p className="text-sm text-slate-600 dark:text-slate-400 mt-0.5">{t.restoreDesc}</p>
        </div>
        <label className="block space-y-1.5 text-xs font-semibold text-slate-700 dark:text-slate-300">
          <span className="block">{t.restorePhrase}</span>
          <textarea
            rows={4}
            autoComplete="off"
            spellCheck={false}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            className={`${inputClass} font-mono`}
            required
          />
        </label>
        {input.trim() !== '' && (
          <div className="text-xs space-y-1">
            <div className={check.countOk ? 'text-emerald-600 dark:text-emerald-400' : 'text-amber-600 dark:text-amber-400'}>
              {check.count} {t.restoreWordCount}
              {!check.countOk && ` · ${t.restoreCountBad}`}
            </div>
            {check.unknown.length > 0 && (
              <div className="text-rose-600 dark:text-rose-400 font-mono">
                {t.restoreUnknown} {check.unknown.map((u) => `#${u.position} "${u.word}"`).join(', ')}
              </div>
            )}
          </div>
        )}
        <PasswordFields password={password} confirm={confirm} onPassword={setPassword} onConfirm={setConfirm} />
        <p className="text-xs text-slate-500 dark:text-slate-400">{t.restoreNote}</p>
        {error && <ErrorBox>{error}</ErrorBox>}
        <div className="flex justify-end">
          <button type="submit" disabled={busy || !ready} className={primaryButton}>
            <KeyRound className="w-4 h-4" />
            {busy ? t.loading : t.restoreSubmit}
          </button>
        </div>
      </form>
    </Card>
  );
}

export function Onboarding() {
  const language = useWallet((s) => s.language);
  const t = TEXT[language];
  const [mode, setMode] = useState<'create' | 'restore'>('create');
  return (
    <div className="space-y-6">
      <PageHeader
        title={t.onboardingTitle}
        subtitle={t.onboardingSubtitle}
        action={
          <div
            role="tablist"
            className="inline-flex p-1 rounded-lg bg-slate-200/80 dark:bg-slate-900 border border-slate-300/80 dark:border-slate-800 shrink-0"
          >
            {(['create', 'restore'] as const).map((m) => (
              <button
                key={m}
                type="button"
                role="tab"
                aria-selected={mode === m}
                onClick={() => setMode(m)}
                className={`px-3.5 py-2 text-xs font-semibold rounded-md transition-colors whitespace-nowrap ${
                  mode === m
                    ? 'bg-white dark:bg-slate-800 text-slate-900 dark:text-emerald-400'
                    : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200'
                }`}
              >
                {m === 'create' ? t.tabCreate : t.tabRestore}
              </button>
            ))}
          </div>
        }
      />
      {mode === 'create' ? <CreateWallet /> : <RestoreWallet />}
    </div>
  );
}
