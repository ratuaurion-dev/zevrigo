/**
 * Terima: alamat penerimaan terakhir, alamat baru, jumlah yang diminta (opsional), URI pembayaran, dan kode QR.
 */

import { Plus, QrCode } from 'lucide-react';
import { useMemo, useState } from 'react';
import { TEXT } from '../lib/i18n';
import { canonicalZvrg, parseZvrg } from '../lib/quanta';
import { paymentUri, qrSvg } from '../lib/qrcode';
import { useWallet } from '../store';
import { Card, CopyButton, ErrorBox, PageHeader, inputClass, primaryButton } from './ui';

export function Receive() {
  const { language, address, newAddress } = useWallet();
  const t = TEXT[language];
  const [amount, setAmount] = useState('');
  const [busy, setBusy] = useState(false);
  const parsed = useMemo(() => (amount.trim() === '' ? null : parseZvrg(amount)), [amount]);
  const uri = paymentUri(address, parsed?.ok ? canonicalZvrg(parsed.quanta) : undefined);
  const qr = useMemo(() => (address ? qrSvg(uri) : null), [address, uri]);

  const makeAddress = async () => {
    setBusy(true);
    try {
      await newAddress();
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="space-y-6">
      <PageHeader
        title={t.receiveTitle}
        subtitle={t.receiveSubtitle}
        action={
          <button type="button" onClick={makeAddress} disabled={busy} className={primaryButton}>
            <Plus className="w-4 h-4" />
            {t.newAddress}
          </button>
        }
      />
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-stretch">
        <Card className="lg:col-span-7 space-y-5">
          <div className="space-y-2">
            <div className="text-xs font-semibold text-slate-700 dark:text-slate-300">{t.receiveAddress}</div>
            <div className="p-3.5 rounded-lg bg-slate-50 dark:bg-slate-950 border border-slate-200 dark:border-slate-800 flex items-center justify-between gap-3">
              <code data-testid="receive-address" className="font-mono text-sm font-semibold text-slate-900 dark:text-slate-100 break-all">
                {address}
              </code>
              <CopyButton text={address} t={t} />
            </div>
            <p className="text-xs text-slate-500 dark:text-slate-400">{t.newAddressHint}</p>
          </div>
          <label className="block space-y-2 text-xs font-semibold text-slate-700 dark:text-slate-300">
            <span className="block">{t.requestAmount}</span>
            <div className="relative">
              <input
                type="text"
                inputMode="decimal"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                placeholder={t.requestAmountPlaceholder}
                className={`${inputClass} pr-16 font-mono`}
              />
              <span className="absolute right-3.5 top-1/2 -translate-y-1/2 text-xs font-mono text-slate-400">ZVRG</span>
            </div>
          </label>
          {parsed && !parsed.ok && <ErrorBox>{t.amountErrors[parsed.error]}</ErrorBox>}
          <div className="space-y-2 pt-4 border-t border-slate-200 dark:border-slate-800">
            <div className="flex items-center justify-between text-xs font-semibold text-slate-700 dark:text-slate-300">
              <span>{t.paymentUri}</span>
              <CopyButton text={uri} t={t} />
            </div>
            <div className="p-3 rounded-lg bg-slate-50 dark:bg-slate-950 border border-slate-200 dark:border-slate-800 font-mono text-xs text-slate-700 dark:text-slate-300 break-all">
              {uri}
            </div>
          </div>
        </Card>
        <Card className="lg:col-span-5 flex flex-col items-center justify-center gap-4">
          <div className="inline-flex items-center gap-1.5 text-xs font-semibold text-slate-700 dark:text-slate-300">
            <QrCode className="w-4 h-4 text-emerald-500" />
            QR
          </div>
          {qr && (
            <div className="p-3.5 bg-white rounded-xl border border-slate-200">
              <svg
                data-testid="receive-qr"
                viewBox={`0 0 ${qr.viewBox} ${qr.viewBox}`}
                className="w-52 h-52"
                shapeRendering="crispEdges"
                role="img"
                aria-label={uri}
              >
                <rect width={qr.viewBox} height={qr.viewBox} fill="#ffffff" />
                <path d={qr.path} fill="#0f172a" />
              </svg>
            </div>
          )}
        </Card>
      </div>
    </div>
  );
}
