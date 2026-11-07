/**
 * Kirim: alamat dan jumlah diperiksa di antarmuka, lalu backend Rust memilih koin, menghitung biaya, menandatangani
 * dengan password wallet, dan mengirim ke node. Biaya pasti ditampilkan dari hasil kirim.
 */

import { CircleCheck, Lock, Send as SendIcon, X } from 'lucide-react';
import { type FormEvent, useMemo, useState } from 'react';
import { checkAddress } from '../lib/address';
import { TEXT, walletErrorText } from '../lib/i18n';
import { type SentDto, ipc, toWalletError } from '../lib/ipc';
import { checkSendAmount, formatQuanta, toQuanta } from '../lib/quanta';
import { useWallet } from '../store';
import { Card, CopyButton, ErrorBox, PageHeader, Zvrg, inputClass, primaryButton, secondaryButton } from './ui';

const FEE_RATES = [1, 2, 5, 10];

export function Send() {
  const { language, network, balance, rpcUrl, refresh, connection } = useWallet();
  const t = TEXT[language];
  const [recipient, setRecipient] = useState('');
  const [amount, setAmount] = useState('');
  const [feeRate, setFeeRate] = useState(1);
  const [confirming, setConfirming] = useState(false);
  const [password, setPassword] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [sent, setSent] = useState<SentDto | null>(null);

  const spendable = balance ? toQuanta(balance.spendable) : 0n;
  const address = useMemo(() => checkAddress(recipient, network), [recipient, network]);
  const parsed = useMemo(() => checkSendAmount(amount, spendable), [amount, spendable]);
  const ready = address.ok && parsed.ok && balance !== null && connection !== 'offline';

  const addressError = () => {
    if (recipient.trim() === '' || address.ok) {
      return null;
    }
    const text = t.addressErrors[address.error];
    return address.detected ? `${text} ${address.detected}` : text;
  };
  const amountError = amount.trim() === '' || parsed.ok ? null : t.amountErrors[parsed.error];

  const review = (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (ready) {
      setPassword('');
      setError(null);
      setConfirming(true);
    }
  };

  const confirm = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!address.ok || !parsed.ok) {
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const result = await ipc.send({
        network,
        rpcUrl: rpcUrl(),
        password,
        to: address.address,
        amount: parsed.quanta.toString(),
        feeRate,
      });
      setSent(result);
      setConfirming(false);
      setPassword('');
      setAmount('');
      setRecipient('');
      await refresh();
    } catch (err) {
      setError(walletErrorText(t, toWalletError(err).code));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="space-y-6">
      <PageHeader
        title={t.sendTitle}
        subtitle={t.sendSubtitle}
        action={
          <div className="text-right text-xs">
            <span className="block text-slate-500 dark:text-slate-400">{t.spendableNow}</span>
            <Zvrg quanta={spendable} lang={language} exact className="text-sm font-bold text-emerald-600 dark:text-emerald-400" />
          </div>
        }
      />

      {sent && (
        <div
          role="status"
          className="p-4 rounded-xl bg-emerald-500/10 border border-emerald-500/30 flex flex-col sm:flex-row sm:items-start justify-between gap-3"
        >
          <div className="space-y-1.5 min-w-0 text-xs">
            <div className="flex items-center gap-2 font-semibold text-emerald-700 dark:text-emerald-300">
              <CircleCheck className="w-4 h-4 shrink-0" />
              {t.sentTitle}: <Zvrg quanta={toQuanta(sent.amount)} lang={language} exact />
            </div>
            <div className="font-mono text-slate-800 dark:text-slate-200 break-all">txid {sent.txid}</div>
            <div className="text-slate-600 dark:text-slate-300">
              {t.sentFee}: {formatQuanta(toQuanta(sent.fee), language)} quanta
              {sent.change !== null && (
                <>
                  {' · '}
                  {t.sentChange}: <Zvrg quanta={toQuanta(sent.change)} lang={language} exact />
                </>
              )}
              {' · '}
              {t.sentInputs}: {sent.inputs} · {t.sentSize}: {sent.size} {t.bytes}
            </div>
          </div>
          <div className="flex items-center gap-2 shrink-0">
            <CopyButton text={sent.txid} t={t} />
            <button
              type="button"
              aria-label={t.cancel}
              onClick={() => setSent(null)}
              className="p-1.5 rounded-lg text-slate-500 hover:text-slate-800 dark:hover:text-slate-200"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
        </div>
      )}

      <Card>
        <form onSubmit={review} className="space-y-5">
          <label className="block space-y-2 text-xs font-semibold text-slate-700 dark:text-slate-300">
            <span className="block">{t.recipient}</span>
            <input
              type="text"
              autoComplete="off"
              spellCheck={false}
              value={recipient}
              onChange={(e) => setRecipient(e.target.value)}
              placeholder={network === 'testnet' ? 'tzvrg1…' : 'rzvrg1…'}
              className={`${inputClass} font-mono`}
              required
            />
          </label>
          {addressError() && <ErrorBox>{addressError()}</ErrorBox>}

          <label className="block space-y-2 text-xs font-semibold text-slate-700 dark:text-slate-300">
            <span className="block">{t.amount}</span>
            <div className="relative">
              <input
                type="text"
                inputMode="decimal"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                placeholder="0"
                className={`${inputClass} pr-16 font-mono`}
                required
              />
              <span className="absolute right-3.5 top-1/2 -translate-y-1/2 text-xs font-mono text-slate-400">ZVRG</span>
            </div>
          </label>
          {parsed.ok && (
            <div className="text-xs font-mono text-slate-500 dark:text-slate-400">
              = {formatQuanta(parsed.quanta, language)} quanta
            </div>
          )}
          {amountError && <ErrorBox>{amountError}</ErrorBox>}

          <div className="space-y-2">
            <div className="text-xs font-semibold text-slate-700 dark:text-slate-300">{t.feeRate}</div>
            <div className="grid grid-cols-4 gap-2">
              {FEE_RATES.map((rate) => (
                <button
                  key={rate}
                  type="button"
                  onClick={() => setFeeRate(rate)}
                  className={`py-2 px-3 text-xs font-mono font-semibold rounded-lg border transition-colors ${
                    feeRate === rate
                      ? 'bg-emerald-600 text-white border-emerald-600'
                      : 'bg-slate-50 dark:bg-slate-950 border-slate-300 dark:border-slate-700 text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800'
                  }`}
                >
                  {rate} {t.feeRateUnit}
                </button>
              ))}
            </div>
            <p className="text-xs text-slate-500 dark:text-slate-400">{t.feeHint}</p>
          </div>

          <button type="submit" disabled={!ready} className={`${primaryButton} w-full`}>
            <SendIcon className="w-4 h-4" />
            {t.review}
          </button>
        </form>
      </Card>

      {confirming && address.ok && parsed.ok && (
        <div
          role="dialog"
          aria-modal="true"
          className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/75"
        >
          <form
            onSubmit={confirm}
            className="w-full max-w-md bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-6 space-y-5"
          >
            <div className="flex items-start justify-between gap-3">
              <h3 className="text-lg font-bold text-slate-900 dark:text-slate-100">{t.confirmTitle}</h3>
              <button
                type="button"
                aria-label={t.cancel}
                onClick={() => setConfirming(false)}
                className="p-1.5 rounded-lg text-slate-400 hover:text-slate-700 dark:hover:text-slate-200"
              >
                <X className="w-4 h-4" />
              </button>
            </div>
            <p className="text-xs text-slate-600 dark:text-slate-400">{t.confirmBody}</p>
            <div className="p-3.5 rounded-lg bg-slate-50 dark:bg-slate-950 border border-slate-200 dark:border-slate-800 space-y-2 text-xs">
              <div>
                <span className="text-slate-500 block">{t.recipient}</span>
                <span className="font-mono text-slate-900 dark:text-slate-100 break-all">{address.address}</span>
              </div>
              <div className="pt-2 border-t border-slate-200 dark:border-slate-800">
                <span className="text-slate-500 block">{t.amount}</span>
                <Zvrg quanta={parsed.quanta} lang={language} exact className="font-bold text-emerald-600 dark:text-emerald-400" />
              </div>
              <div className="pt-2 border-t border-slate-200 dark:border-slate-800">
                <span className="text-slate-500 block">{t.feeRate}</span>
                <span className="font-mono">
                  {feeRate} {t.feeRateUnit}
                </span>
              </div>
            </div>
            <label className="block space-y-1.5 text-xs font-semibold text-slate-700 dark:text-slate-300">
              <span className="block">{t.password}</span>
              <input
                type="password"
                autoComplete="current-password"
                autoFocus
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className={inputClass}
                required
              />
            </label>
            {error && <ErrorBox>{error}</ErrorBox>}
            <div className="flex items-center justify-end gap-2.5">
              <button type="button" onClick={() => setConfirming(false)} className={secondaryButton}>
                {t.cancel}
              </button>
              <button type="submit" disabled={busy || password === ''} className={primaryButton}>
                <Lock className="w-4 h-4" />
                {busy ? t.sending : t.signAndSend}
              </button>
            </div>
          </form>
        </div>
      )}
    </div>
  );
}
