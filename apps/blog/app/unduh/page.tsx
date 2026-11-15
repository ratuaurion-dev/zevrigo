import React from 'react';
import type { Metadata } from 'next';
import Link from 'next/link';
import { ArrowUpRight, Download } from 'lucide-react';
import { RELEASES_URL, UNDUHAN, type Unduhan } from '@/lib/unduhan';

export const metadata: Metadata = {
  title: 'Unduh wallet',
  description:
    'Unduh wallet Zevrigo untuk jaringan demo regtest: installer Windows dari GitHub Releases, dengan cara memeriksa SHA-256.',
};

const MUTED = 'text-[#52525B] dark:text-[#A1A1AA]';
const BORDER = 'border-[#E4E4E7] dark:border-[#27272A]';
const LINK = 'text-[#0F766E] dark:text-[#2DD4BF] underline underline-offset-4 decoration-[#0F766E]/40 dark:decoration-[#2DD4BF]/40';

function Code({ children }: { children: React.ReactNode }) {
  return (
    <pre className="overflow-x-auto bg-[#18181B] dark:bg-[#0D0D10] text-[#FAF9F6] p-4 text-xs md:text-[13px] font-mono leading-relaxed">
      <code>{children}</code>
    </pre>
  );
}

function Card({ item }: { item: Unduhan }) {
  const file = item.berkas;
  return (
    <section className={`border ${BORDER} bg-[#FAF9F6] dark:bg-[#121215] p-6 md:p-8 space-y-5`}>
      <div className="flex flex-wrap items-baseline justify-between gap-3">
        <h2 className="font-serif text-2xl font-semibold">
          {item.platform} <span className={`font-sans text-sm font-normal ${MUTED}`}>versi {item.versi}</span>
        </h2>
        <span
          className={`text-xs font-semibold ${
            item.status === 'tersedia' ? 'text-[#0F766E] dark:text-[#2DD4BF]' : 'text-amber-700 dark:text-amber-400'
          }`}
        >
          {item.status === 'tersedia' ? 'Tersedia' : 'Segera'}
        </span>
      </div>
      <p className={`text-sm ${MUTED}`}>{item.syarat}</p>
      {file && (
        <div className="space-y-4">
          <a
            href={file.url}
            rel="noopener noreferrer"
            className="inline-flex items-center gap-2 bg-[#0F766E] hover:bg-[#115E59] dark:bg-[#2DD4BF] dark:hover:bg-[#5EEAD4] text-[#FAF9F6] dark:text-[#09090B] px-5 py-3 text-sm font-semibold"
          >
            <Download className="h-4 w-4" aria-hidden="true" />
            Unduh {file.nama}
          </a>
          <div className="space-y-2 text-sm">
            <div className="font-semibold">Sidik SHA-256</div>
            {file.sha256 ? (
              <code className="block break-all bg-[#F4F3EE] dark:bg-[#18181B] border border-[#E4E4E7] dark:border-[#27272A] px-3 py-2 font-mono text-xs">
                {file.sha256}
              </code>
            ) : (
              <p className={MUTED}>
                Bandingkan dengan isi berkas{' '}
                <a href={file.sha256Sums} rel="noopener noreferrer" className={LINK}>
                  SHA256SUMS.txt
                </a>{' '}
                di halaman rilis.
              </p>
            )}
          </div>
          <div className="space-y-2 text-sm">
            <div className="font-semibold">Periksa berkas sebelum memasang (PowerShell)</div>
            <Code>{`Get-FileHash .\\${file.nama} -Algorithm SHA256`}</Code>
            <p className={MUTED}>
              Angka yang keluar harus sama persis. Kalau berbeda, hapus berkasnya dan jangan dipasang.
            </p>
          </div>
          <a href={file.halamanRilis} rel="noopener noreferrer" className={`inline-flex items-center gap-1 text-sm ${LINK}`}>
            Halaman rilis di GitHub
            <ArrowUpRight className="h-3.5 w-3.5" aria-hidden="true" />
          </a>
        </div>
      )}
      <ul className={`list-disc pl-5 space-y-1.5 text-sm leading-relaxed ${MUTED}`}>
        {item.catatan.map((line) => (
          <li key={line}>{line}</li>
        ))}
      </ul>
    </section>
  );
}

export default function DownloadPage() {
  return (
    <div className="pt-10 md:pt-14 mx-auto max-w-3xl space-y-8">
      <header className="space-y-4">
        <h1 className="font-serif text-3xl md:text-5xl font-semibold leading-tight tracking-tight">Unduh wallet Zevrigo</h1>
        <p className={`text-base md:text-lg leading-relaxed ${MUTED}`}>
          Wallet untuk jaringan demo regtest. Berkasnya disimpan di{' '}
          <a href={RELEASES_URL} rel="noopener noreferrer" className={LINK}>
            GitHub Releases
          </a>{' '}
          proyek Zevrigo dan dibangun dari kode sumber yang terbuka. Unduh hanya dari halaman ini atau dari GitHub
          ratuaurion-dev/zevrigo.
        </p>
      </header>
      {UNDUHAN.map((item) => (
        <Card key={item.id} item={item} />
      ))}
      <p className={`text-sm leading-relaxed ${MUTED}`}>
        Baru pertama kali? Ikuti{' '}
        <Link href="/artikel/panduan-wallet-dan-faucet/" className={LINK}>
          panduan mencoba wallet dan faucet
        </Link>
        . Kabar versi Android ada di{' '}
        <Link href="/artikel/kabar-wallet-android/" className={LINK}>
          artikel wallet Android
        </Link>
        .
      </p>
    </div>
  );
}
