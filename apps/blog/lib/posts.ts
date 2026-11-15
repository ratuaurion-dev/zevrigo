/**
 * Artikel blog (TSK-M64): satu berkas Markdown per artikel di `posts/<slug>.md`, dibaca saat `next build`. Menerbitkan
 * artikel = commit berkas baru; tidak ada halaman admin atau penyimpanan di server.
 *
 * Front matter di antara dua baris `---`, satu `kunci: nilai` per baris:
 * `judul`, `ringkasan`, `tanggal` (YYYY-MM-DD), `kategori`, `penulis`, opsional `sampul` (`/sampul/<nama>.jpg`) dan
 * `unggulan` (`ya`). Front matter yang tidak lengkap atau tidak sah menghentikan build.
 */
import fs from 'node:fs';
import path from 'node:path';
import { type Block, parseMarkdown, plainText } from './markdown';

export const CATEGORIES = ['Pengumuman', 'Panduan', 'Protokol', 'Tokenomik'] as const;
export type Category = (typeof CATEGORIES)[number];

export type Post = {
  slug: string;
  title: string;
  summary: string;
  date: string;
  category: Category;
  author: string;
  cover: string | null;
  featured: boolean;
  blocks: Block[];
  readingMinutes: number;
};

const SLUG = /^[a-z0-9]+(-[a-z0-9]+)*$/;
const DATE = /^\d{4}-(0[1-9]|1[0-2])-(0[1-9]|[12]\d|3[01])$/;
const COVER = /^\/sampul\/[a-z0-9]+(-[a-z0-9]+)*\.jpg$/;
const KEYS = ['judul', 'ringkasan', 'tanggal', 'kategori', 'penulis', 'sampul', 'unggulan'];
const MONTHS = [
  'Januari',
  'Februari',
  'Maret',
  'April',
  'Mei',
  'Juni',
  'Juli',
  'Agustus',
  'September',
  'Oktober',
  'November',
  'Desember',
];

/** Front matter `---` ... `---` di awal berkas dan badan sesudahnya. */
export function parseFrontMatter(source: string): { meta: Record<string, string>; body: string } {
  const text = source.replace(/\r\n?/g, '\n');
  if (!text.startsWith('---\n')) {
    throw new Error('front matter tidak ada (baris pertama harus ---)');
  }
  const end = text.indexOf('\n---\n', 4);
  if (end < 0) {
    throw new Error('front matter tidak ditutup ---');
  }
  const meta: Record<string, string> = {};
  for (const line of text.slice(4, end).split('\n')) {
    if (!line.trim()) continue;
    const colon = line.indexOf(':');
    if (colon <= 0) throw new Error(`baris front matter tidak sah: ${line}`);
    const key = line.slice(0, colon).trim();
    if (!KEYS.includes(key)) throw new Error(`kunci front matter tidak dikenal: ${key}`);
    if (key in meta) throw new Error(`kunci front matter ganda: ${key}`);
    meta[key] = line.slice(colon + 1).trim();
  }
  return { meta, body: text.slice(end + 5) };
}

/** Tanggal `YYYY-MM-DD` dalam bahasa Indonesia, misalnya `14 November 2026`. */
export function formatDate(iso: string): string {
  if (!DATE.test(iso)) throw new Error(`tanggal tidak sah: ${iso}`);
  const [year, month, day] = iso.split('-').map(Number);
  return `${day} ${MONTHS[month - 1]} ${year}`;
}

/** Satu artikel dari isi berkas `posts/<slug>.md`. */
export function parsePost(slug: string, source: string): Post {
  if (!SLUG.test(slug)) throw new Error(`nama berkas artikel tidak sah: ${slug}`);
  const { meta, body } = parseFrontMatter(source);
  const need = (key: string): string => {
    const value = meta[key];
    if (!value) throw new Error(`${slug}: front matter ${key} wajib diisi`);
    return value;
  };
  const category = need('kategori');
  if (!(CATEGORIES as readonly string[]).includes(category)) {
    throw new Error(`${slug}: kategori tidak dikenal: ${category}`);
  }
  const date = need('tanggal');
  formatDate(date);
  const cover = meta.sampul ?? null;
  if (cover !== null && !COVER.test(cover)) throw new Error(`${slug}: sampul tidak sah: ${cover}`);
  const featured = meta.unggulan === undefined ? false : meta.unggulan === 'ya';
  if (meta.unggulan !== undefined && meta.unggulan !== 'ya') {
    throw new Error(`${slug}: unggulan hanya boleh ya`);
  }
  const blocks = parseMarkdown(body);
  if (blocks.length === 0) throw new Error(`${slug}: badan artikel kosong`);
  const words = plainText(blocks).split(/\s+/).filter(Boolean).length;
  return {
    slug,
    title: need('judul'),
    summary: need('ringkasan'),
    date,
    category: category as Category,
    author: need('penulis'),
    cover,
    featured,
    blocks,
    readingMinutes: Math.max(1, Math.ceil(words / 200)),
  };
}

/** Semua artikel di `dir` (bawaan `posts/` di folder blog), terbaru dulu; tanggal sama diurutkan menurut slug. */
export function loadPosts(dir: string = path.join(process.cwd(), 'posts')): Post[] {
  const files = fs.readdirSync(dir).filter((name) => name.endsWith('.md'));
  const posts = files.map((name) =>
    parsePost(name.slice(0, -3), fs.readFileSync(path.join(dir, name), 'utf8'))
  );
  return posts.sort((a, b) => (a.date === b.date ? a.slug.localeCompare(b.slug) : b.date.localeCompare(a.date)));
}

/** Artikel unggulan terbaru (atau artikel terbaru bila tidak ada yang unggulan) dan sisanya. */
export function splitFeatured(posts: Post[]): { featured: Post | null; rest: Post[] } {
  const featured = posts.find((post) => post.featured) ?? posts[0] ?? null;
  return { featured, rest: posts.filter((post) => post !== featured) };
}
