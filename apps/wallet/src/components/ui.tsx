/**
 * Komponen kecil yang dipakai semua layar: kartu, judul halaman, kotak galat, tombol salin, dan jumlah ZVRG.
 */

import { Check, Copy, TriangleAlert } from 'lucide-react';
import { type ReactNode, useState } from 'react';
import { formatQuanta, formatZvrg, type Lang } from '../lib/quanta';
import type { Dict } from '../lib/i18n';

export const inputClass =
  'w-full px-3.5 py-2.5 text-sm rounded-lg bg-slate-50 dark:bg-slate-950 border border-slate-300 dark:border-slate-700 text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-2 focus:ring-emerald-500';

export const primaryButton =
  'inline-flex items-center justify-center gap-2 px-4 py-2.5 text-sm font-semibold rounded-lg bg-emerald-600 hover:bg-emerald-500 disabled:opacity-40 disabled:cursor-not-allowed text-white transition-colors whitespace-nowrap';

export const secondaryButton =
  'inline-flex items-center justify-center gap-2 px-3.5 py-2 text-xs font-semibold rounded-lg border border-slate-300 dark:border-slate-700 text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800 disabled:opacity-40 transition-colors whitespace-nowrap';

export function Card({
  children,
  className = '',
  flush = false,
}: {
  children: ReactNode;
  className?: string;
  flush?: boolean;
}) {
  return (
    <div
      className={`bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl ${flush ? 'overflow-hidden' : 'p-5'} ${className}`}
    >
      {children}
    </div>
  );
}

export function PageHeader({ title, subtitle, action }: { title: string; subtitle?: string; action?: ReactNode }) {
  return (
    <div className="flex flex-col sm:flex-row sm:items-end justify-between gap-4 border-b border-slate-200 dark:border-slate-800 pb-5">
      <div>
        <h1 className="text-2xl font-bold tracking-tight text-slate-900 dark:text-slate-100">{title}</h1>
        {subtitle && <p className="text-sm text-slate-600 dark:text-slate-400 mt-1 max-w-2xl">{subtitle}</p>}
      </div>
      {action}
    </div>
  );
}

export function ErrorBox({ children }: { children: ReactNode }) {
  return (
    <div
      role="alert"
      className="flex items-start gap-2 p-3 rounded-lg bg-rose-500/10 border border-rose-500/30 text-xs text-rose-700 dark:text-rose-300"
    >
      <TriangleAlert className="w-4 h-4 shrink-0 mt-0.5" />
      <div>{children}</div>
    </div>
  );
}

export function CopyButton({ text, t, label }: { text: string; t: Dict; label?: string }) {
  const [copied, setCopied] = useState(false);
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 1800);
    } catch {
      // Papan klip tidak tersedia.
    }
  };
  return (
    <button type="button" onClick={copy} className={secondaryButton} aria-label={label ?? t.copy}>
      {copied ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Copy className="w-3.5 h-3.5" />}
      {copied ? t.copied : (label ?? t.copy)}
    </button>
  );
}

/**
 * Jumlah ZVRG. Ringkas: maksimal 4 angka di belakang koma (dipotong, ditandai "…"); `exact` menampilkan semua angka.
 * Judul (hover) selalu berisi nilai tepat dalam ZVRG dan quanta.
 */
export function Zvrg({
  quanta,
  lang,
  exact = false,
  signed = false,
  className = '',
}: {
  quanta: bigint;
  lang: Lang;
  exact?: boolean;
  signed?: boolean;
  className?: string;
}) {
  const shown = formatZvrg(quanta, lang, exact ? undefined : 4);
  const full = formatZvrg(quanta, lang).text;
  const sign = signed && quanta > 0n ? '+' : '';
  return (
    <span className={`font-mono tabular-nums ${className}`} title={`${full} ZVRG = ${formatQuanta(quanta, lang)} quanta`}>
      {sign}
      {shown.text}
      {shown.cut ? '…' : ''} ZVRG
    </span>
  );
}
