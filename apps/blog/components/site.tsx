import React from 'react';
import Link from 'next/link';
import { ArrowUpRight } from 'lucide-react';
import { formatDate, type Post } from '@/lib/posts';

export const EXPLORER_URL = 'https://zevrigo.ratuaurion.store';
export const FAUCET_URL = 'https://zevrigo.ratuaurion.store/faucet/';
export const REPO_URL = 'https://github.com/ratuaurion-dev/zevrigo';

const MUTED = 'text-[#52525B] dark:text-[#A1A1AA]';
const BORDER = 'border-[#E4E4E7] dark:border-[#27272A]';

function ExternalLink({ href, children }: { href: string; children: React.ReactNode }) {
  return (
    <a
      href={href}
      rel="noopener noreferrer"
      className="hover:text-[#0F766E] dark:hover:text-[#2DD4BF] transition-colors whitespace-nowrap"
    >
      {children}
    </a>
  );
}

export function SiteHeader() {
  return (
    <header className={`border-b ${BORDER} bg-[#FAF9F6] dark:bg-[#09090B]`}>
      <div className="mx-auto max-w-6xl flex flex-wrap items-center justify-between gap-x-8 gap-y-3 px-5 md:px-8 py-4">
        <Link href="/" className="flex items-baseline gap-2">
          <span className="font-serif text-xl font-semibold tracking-tight text-[#18181B] dark:text-[#F4F4F5]">
            Zevrigo
          </span>
          <span className="font-mono text-xs text-[#0F766E] dark:text-[#2DD4BF]">Blog</span>
        </Link>
        <nav aria-label="Navigasi" className={`flex flex-wrap items-center gap-x-6 gap-y-2 text-sm ${MUTED}`}>
          <Link href="/" className="hover:text-[#0F766E] dark:hover:text-[#2DD4BF] transition-colors">
            Artikel
          </Link>
          <Link href="/unduh/" className="hover:text-[#0F766E] dark:hover:text-[#2DD4BF] transition-colors">
            Unduh
          </Link>
          <ExternalLink href={EXPLORER_URL}>Explorer</ExternalLink>
          <ExternalLink href={FAUCET_URL}>Faucet</ExternalLink>
          <ExternalLink href={REPO_URL}>GitHub</ExternalLink>
        </nav>
      </div>
    </header>
  );
}

/** Pita peringatan jaringan demo, di setiap halaman. */
export function DemoNotice() {
  return (
    <div role="note" className="border-b border-amber-500/40 bg-amber-500/10 px-5 md:px-8 py-2.5">
      <p className="mx-auto max-w-6xl text-xs leading-relaxed text-amber-900 dark:text-amber-200">
        <strong className="font-semibold">Jaringan demo regtest.</strong> Koin ZVRG di jaringan demo tidak bernilai, tidak
        dijual, dan tidak ditukar ke mainnet. Chain demo bisa direset. Mainnet belum ada.
      </p>
    </div>
  );
}

export function SiteFooter() {
  return (
    <footer className={`mt-20 border-t ${BORDER}`}>
      <div className={`mx-auto max-w-6xl grid gap-6 md:grid-cols-2 px-5 md:px-8 py-10 text-xs leading-relaxed ${MUTED}`}>
        <div className="space-y-2">
          <div className="font-serif text-base font-semibold text-[#18181B] dark:text-[#F4F4F5]">Zevrigo (ZVRG)</div>
          <p>
            Blockchain Proof-of-Work dengan model UTXO, ditulis dalam Rust. Kode sumber terbuka di GitHub dengan lisensi MIT
            atau Apache-2.0.
          </p>
        </div>
        <div className="space-y-2 md:text-right">
          <p>Tanpa kuki, tanpa analitik, tanpa font atau skrip dari pihak lain.</p>
          <p className="space-x-4">
            <Link href="/unduh/" className="hover:text-[#0F766E] dark:hover:text-[#2DD4BF] transition-colors">
              Unduh wallet
            </Link>
            <ExternalLink href={EXPLORER_URL}>Explorer</ExternalLink>
            <ExternalLink href={FAUCET_URL}>Faucet demo</ExternalLink>
            <ExternalLink href={REPO_URL}>Kode sumber</ExternalLink>
          </p>
        </div>
      </div>
    </footer>
  );
}

/** Baris kecil: kategori · tanggal · menit baca. */
export function Kicker({ post }: { post: Post }) {
  return (
    <div className={`flex flex-wrap items-center gap-2 text-xs ${MUTED}`}>
      <span className="font-semibold text-[#0F766E] dark:text-[#2DD4BF]">{post.category}</span>
      <span aria-hidden="true">·</span>
      <time dateTime={post.date}>{formatDate(post.date)}</time>
      <span aria-hidden="true">·</span>
      <span>{post.readingMinutes} menit baca</span>
    </div>
  );
}

/** Sampul artikel, atau bidang tipografis bila artikel tanpa sampul. */
export function Cover({ post, className = '' }: { post: Post; className?: string }) {
  if (post.cover) {
    return (
      // eslint-disable-next-line @next/next/no-img-element
      <img src={post.cover} alt="" className={`h-full w-full object-cover ${className}`} loading="lazy" decoding="async" />
    );
  }
  return (
    <div
      aria-hidden="true"
      className={`flex h-full w-full items-end bg-[#F4F3EE] dark:bg-[#18181B] p-6 ${className}`}
    >
      <span className="font-serif text-3xl md:text-4xl font-semibold leading-none text-[#0F766E]/25 dark:text-[#2DD4BF]/20">
        {post.category}
      </span>
    </div>
  );
}

export function FeaturedPost({ post }: { post: Post }) {
  return (
    <article className={`border ${BORDER} bg-[#FAF9F6] dark:bg-[#121215] overflow-hidden`}>
      <div className="grid grid-cols-1 lg:grid-cols-12 items-stretch">
        <div className="lg:col-span-7 flex flex-col justify-between p-6 md:p-10 gap-8">
          <div className="space-y-4">
            <Kicker post={post} />
            <h2 className="font-serif text-2xl md:text-4xl font-semibold leading-tight tracking-tight text-[#18181B] dark:text-[#F4F4F5]">
              <Link
                href={`/artikel/${post.slug}/`}
                className="hover:text-[#0F766E] dark:hover:text-[#2DD4BF] transition-colors"
              >
                {post.title}
              </Link>
            </h2>
            <p className={`text-base leading-relaxed max-w-2xl ${MUTED}`}>{post.summary}</p>
          </div>
          <div className={`pt-5 border-t ${BORDER} flex flex-wrap items-center justify-between gap-4 text-xs ${MUTED}`}>
            <span className="font-medium text-[#18181B] dark:text-[#F4F4F5]">{post.author}</span>
            <Link
              href={`/artikel/${post.slug}/`}
              className="inline-flex items-center gap-1.5 font-semibold text-[#0F766E] dark:text-[#2DD4BF] hover:underline underline-offset-4"
            >
              Baca artikel
              <ArrowUpRight className="h-3.5 w-3.5" aria-hidden="true" />
            </Link>
          </div>
        </div>
        <div className={`lg:col-span-5 aspect-[16/9] lg:aspect-auto border-t lg:border-t-0 lg:border-l ${BORDER}`}>
          <Cover post={post} />
        </div>
      </div>
    </article>
  );
}

export function PostCard({ post }: { post: Post }) {
  return (
    <article className={`flex flex-col justify-between border ${BORDER} bg-[#FAF9F6] dark:bg-[#121215] overflow-hidden`}>
      <div>
        <div className={`aspect-[16/10] border-b ${BORDER}`}>
          <Cover post={post} />
        </div>
        <div className="p-6 space-y-3">
          <Kicker post={post} />
          <h3 className="font-serif text-xl font-semibold leading-snug text-[#18181B] dark:text-[#F4F4F5]">
            <Link
              href={`/artikel/${post.slug}/`}
              className="hover:text-[#0F766E] dark:hover:text-[#2DD4BF] transition-colors"
            >
              {post.title}
            </Link>
          </h3>
          <p className={`text-sm leading-relaxed ${MUTED}`}>{post.summary}</p>
        </div>
      </div>
      <div className={`mx-6 mb-6 pt-4 border-t ${BORDER} flex items-center justify-between gap-2 text-xs ${MUTED}`}>
        <span className="font-medium text-[#18181B] dark:text-[#F4F4F5] truncate">{post.author}</span>
        <Link
          href={`/artikel/${post.slug}/`}
          className="inline-flex items-center gap-1 font-semibold text-[#0F766E] dark:text-[#2DD4BF] hover:underline underline-offset-4 whitespace-nowrap"
        >
          Baca
          <ArrowUpRight className="h-3.5 w-3.5" aria-hidden="true" />
        </Link>
      </div>
    </article>
  );
}
