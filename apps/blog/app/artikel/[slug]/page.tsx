import React from 'react';
import type { Metadata } from 'next';
import Link from 'next/link';
import { notFound } from 'next/navigation';
import { ArrowLeft } from 'lucide-react';
import { Markdown } from '@/components/markdown';
import { Cover, Kicker } from '@/components/site';
import { loadPosts } from '@/lib/posts';

export const dynamicParams = false;

type Params = Promise<{ slug: string }>;

export function generateStaticParams(): { slug: string }[] {
  return loadPosts().map((post) => ({ slug: post.slug }));
}

export async function generateMetadata({ params }: { params: Params }): Promise<Metadata> {
  const { slug } = await params;
  const post = loadPosts().find((item) => item.slug === slug);
  if (!post) return {};
  return {
    title: post.title,
    description: post.summary,
    authors: [{ name: post.author }],
    openGraph: {
      title: post.title,
      description: post.summary,
      type: 'article',
      publishedTime: post.date,
      authors: [post.author],
      ...(post.cover ? { images: [post.cover] } : {}),
    },
  };
}

export default async function ArticlePage({ params }: { params: Params }) {
  const { slug } = await params;
  const post = loadPosts().find((item) => item.slug === slug);
  if (!post) notFound();
  return (
    <article className="pt-8 md:pt-12 mx-auto max-w-3xl">
      <nav aria-label="Jejak" className="text-xs text-[#52525B] dark:text-[#A1A1AA]">
        <Link href="/" className="hover:text-[#0F766E] dark:hover:text-[#2DD4BF]">
          Artikel
        </Link>
        <span aria-hidden="true" className="mx-2">
          /
        </span>
        <span>{post.category}</span>
      </nav>
      <header className="mt-6 space-y-5">
        <Kicker post={post} />
        <h1 className="font-serif text-3xl md:text-[44px] font-semibold leading-tight tracking-tight">{post.title}</h1>
        <p className="text-lg leading-relaxed text-[#52525B] dark:text-[#A1A1AA]">{post.summary}</p>
        <div className="text-sm">
          Oleh <span className="font-semibold">{post.author}</span>
        </div>
      </header>
      {post.cover && (
        <div className="mt-8 aspect-[16/9] border border-[#E4E4E7] dark:border-[#27272A] overflow-hidden">
          <Cover post={post} />
        </div>
      )}
      <div className="mt-8">
        <Markdown blocks={post.blocks} />
      </div>
      <div className="mt-14 pt-6 border-t border-[#E4E4E7] dark:border-[#27272A]">
        <Link
          href="/"
          className="inline-flex items-center gap-2 text-sm font-semibold text-[#0F766E] dark:text-[#2DD4BF] hover:underline underline-offset-4"
        >
          <ArrowLeft className="h-4 w-4" aria-hidden="true" />
          Semua artikel
        </Link>
      </div>
    </article>
  );
}
