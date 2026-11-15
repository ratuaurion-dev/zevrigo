import React from 'react';
import Link from 'next/link';

export default function NotFound() {
  return (
    <div className="py-24 text-center space-y-4">
      <div className="font-mono text-xs text-[#52525B] dark:text-[#A1A1AA]">HTTP 404</div>
      <h1 className="font-serif text-3xl font-semibold">Halaman tidak ditemukan</h1>
      <p className="text-sm text-[#52525B] dark:text-[#A1A1AA]">Alamat ini tidak ada di blog Zevrigo.</p>
      <Link href="/" className="inline-block text-sm font-semibold text-[#0F766E] dark:text-[#2DD4BF] hover:underline underline-offset-4">
        Kembali ke semua artikel
      </Link>
    </div>
  );
}
