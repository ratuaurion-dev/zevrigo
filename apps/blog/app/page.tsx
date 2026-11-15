import React from 'react';
import { FeaturedPost, PostCard } from '@/components/site';
import { loadPosts, splitFeatured } from '@/lib/posts';

export default function HomePage() {
  const { featured, rest } = splitFeatured(loadPosts());
  return (
    <div className="pt-10 md:pt-14 space-y-12">
      <section className="max-w-3xl space-y-4">
        <h1 className="font-serif text-3xl md:text-5xl font-semibold leading-tight tracking-tight">
          Catatan pengembangan Zevrigo
        </h1>
        <p className="text-base md:text-lg leading-relaxed text-[#52525B] dark:text-[#A1A1AA]">
          Zevrigo (ZVRG) adalah blockchain Proof-of-Work dengan model UTXO yang ditulis dalam Rust. Di sini kami menulis
          apa yang sudah berjalan, apa yang sedang diuji, dan angka yang bisa Anda periksa sendiri di chain.
        </p>
      </section>
      {featured && <FeaturedPost post={featured} />}
      {rest.length > 0 && (
        <section aria-label="Artikel lain" className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {rest.map((post) => (
            <PostCard key={post.slug} post={post} />
          ))}
        </section>
      )}
    </div>
  );
}
