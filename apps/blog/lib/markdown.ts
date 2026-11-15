/**
 * Pengurai Markdown kecil untuk artikel blog (TSK-M64), tanpa paket tambahan. Hanya bagian yang dipakai artikel:
 * judul `##` dan `###`, paragraf, daftar `-` dan `1.`, blok kode berpagar, kutipan `>`, tabel GFM, garis `---`, serta
 * teks sebaris `kode`, **tebal**, dan [tautan](https://...). HTML mentah tidak pernah diteruskan: semua teks dirender
 * React sebagai teks biasa. Tautan hanya `https://`, jalur situs sendiri (`/...`), atau jangkar (`#...`).
 */

export type Inline =
  | { kind: 'text'; text: string }
  | { kind: 'code'; text: string }
  | { kind: 'strong'; text: string }
  | { kind: 'link'; href: string; text: string };

export type Block =
  | { kind: 'heading'; level: 2 | 3; id: string; text: string }
  | { kind: 'paragraph'; inline: Inline[] }
  | { kind: 'list'; ordered: boolean; items: Inline[][] }
  | { kind: 'code'; language: string; code: string }
  | { kind: 'quote'; inline: Inline[] }
  | { kind: 'table'; header: Inline[][]; rows: Inline[][][] }
  | { kind: 'rule' };

/** Jangkar judul: huruf kecil tanpa diakritik, selain huruf dan angka menjadi `-`. */
export function slugify(text: string): string {
  return text
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
}

/** `true` untuk tautan yang boleh dirender: `https://`, jalur situs sendiri, atau jangkar. */
export function isSafeHref(href: string): boolean {
  return /^https:\/\/[^\s"'<>]+$/.test(href) || /^\/(?!\/)[^\s"'<>]*$/.test(href) || /^#[a-z0-9-]+$/.test(href);
}

const INLINE = /(`[^`]+`|\*\*[^*]+\*\*|\[[^\]]+\]\([^)\s]+\))/g;

/** Teks sebaris: `kode`, **tebal**, [tautan](href). Tautan yang tidak aman menjadi teks biasa. */
export function parseInline(text: string): Inline[] {
  const out: Inline[] = [];
  for (const token of text.split(INLINE)) {
    if (!token) continue;
    if (token.length > 2 && token.startsWith('`') && token.endsWith('`')) {
      out.push({ kind: 'code', text: token.slice(1, -1) });
    } else if (token.length > 4 && token.startsWith('**') && token.endsWith('**')) {
      out.push({ kind: 'strong', text: token.slice(2, -2) });
    } else if (token.startsWith('[') && token.endsWith(')') && token.includes('](')) {
      const split = token.indexOf('](');
      const label = token.slice(1, split);
      const href = token.slice(split + 2, -1);
      out.push(isSafeHref(href) ? { kind: 'link', href, text: label } : { kind: 'text', text: label });
    } else {
      out.push({ kind: 'text', text: token });
    }
  }
  return out;
}

function tableCells(line: string): string[] {
  const trimmed = line.trim().replace(/^\|/, '').replace(/\|$/, '');
  return trimmed.split('|').map((cell) => cell.trim());
}

const TABLE_RULE = /^\|?\s*:?-{3,}:?\s*(\|\s*:?-{3,}:?\s*)*\|?$/;
const ORDERED = /^\d+\.\s+/;

function startsBlock(line: string, next: string | undefined): boolean {
  return (
    line.startsWith('## ') ||
    line.startsWith('### ') ||
    line.startsWith('```') ||
    line.startsWith('> ') ||
    line.startsWith('- ') ||
    ORDERED.test(line) ||
    line === '---' ||
    (line.startsWith('|') && next !== undefined && TABLE_RULE.test(next.trim()))
  );
}

/** Mengurai badan artikel (tanpa front matter) menjadi blok. */
export function parseMarkdown(source: string): Block[] {
  const lines = source.replace(/\r\n?/g, '\n').split('\n');
  const blocks: Block[] = [];
  const ids = new Map<string, number>();
  const uniqueId = (text: string): string => {
    const base = slugify(text) || 'bagian';
    const seen = ids.get(base) ?? 0;
    ids.set(base, seen + 1);
    return seen === 0 ? base : `${base}-${seen + 1}`;
  };

  let i = 0;
  while (i < lines.length) {
    const line = lines[i].trim();
    const next = lines[i + 1];
    if (!line) {
      i += 1;
      continue;
    }
    if (line.startsWith('```')) {
      const language = line.slice(3).trim();
      const code: string[] = [];
      i += 1;
      while (i < lines.length && !lines[i].trim().startsWith('```')) {
        code.push(lines[i]);
        i += 1;
      }
      i += 1;
      blocks.push({ kind: 'code', language, code: code.join('\n') });
      continue;
    }
    if (line.startsWith('### ') || line.startsWith('## ')) {
      const level = line.startsWith('### ') ? 3 : 2;
      const text = line.slice(level + 1).trim();
      blocks.push({ kind: 'heading', level, id: uniqueId(text), text });
      i += 1;
      continue;
    }
    if (line === '---') {
      blocks.push({ kind: 'rule' });
      i += 1;
      continue;
    }
    if (line.startsWith('> ')) {
      const parts: string[] = [];
      while (i < lines.length && lines[i].trim().startsWith('> ')) {
        parts.push(lines[i].trim().slice(2).trim());
        i += 1;
      }
      blocks.push({ kind: 'quote', inline: parseInline(parts.join(' ')) });
      continue;
    }
    if (line.startsWith('|') && next !== undefined && TABLE_RULE.test(next.trim())) {
      const header = tableCells(line).map(parseInline);
      const rows: Inline[][][] = [];
      i += 2;
      while (i < lines.length && lines[i].trim().startsWith('|')) {
        rows.push(tableCells(lines[i]).map(parseInline));
        i += 1;
      }
      blocks.push({ kind: 'table', header, rows });
      continue;
    }
    if (line.startsWith('- ') || ORDERED.test(line)) {
      const ordered = !line.startsWith('- ');
      const items: Inline[][] = [];
      while (i < lines.length) {
        const item = lines[i].trim();
        if (ordered ? !ORDERED.test(item) : !item.startsWith('- ')) break;
        items.push(parseInline(ordered ? item.replace(ORDERED, '') : item.slice(2)));
        i += 1;
      }
      blocks.push({ kind: 'list', ordered, items });
      continue;
    }
    const parts: string[] = [];
    while (i < lines.length) {
      const current = lines[i].trim();
      if (!current || (parts.length > 0 && startsBlock(current, lines[i + 1]))) break;
      parts.push(current);
      i += 1;
    }
    blocks.push({ kind: 'paragraph', inline: parseInline(parts.join(' ')) });
  }
  return blocks;
}

/** Teks polos semua blok (untuk menghitung kata). */
export function plainText(blocks: Block[]): string {
  const inline = (items: Inline[]) => items.map((item) => item.text).join('');
  return blocks
    .map((block) => {
      switch (block.kind) {
        case 'heading':
          return block.text;
        case 'paragraph':
        case 'quote':
          return inline(block.inline);
        case 'list':
          return block.items.map(inline).join(' ');
        case 'code':
          return block.code;
        case 'table':
          return [block.header, ...block.rows].map((row) => row.map(inline).join(' ')).join(' ');
        case 'rule':
          return '';
      }
    })
    .join(' ');
}
