// Pemilihan bahasa dan pengisian teks (Spesifikasi Explorer Web v0.1 Bagian 3).
import { en } from './en';
import { id } from './id';
import type { Dict } from './id';

export type { Dict } from './id';

export type Lang = 'id' | 'en';

/** Kunci localStorage pilihan bahasa (hanya di browser, tidak pernah dikirim ke server). */
export const STORAGE_KEY = 'hsis-explorer-lang';

/** Kamus untuk `lang`. */
export function dict(lang: Lang): Dict {
  return lang === 'en' ? en : id;
}

/** Bahasa awal: pilihan tersimpan, lalu bahasa browser berawalan `en`, selain itu Indonesia. */
export function detectLanguage(stored: string | null, browser: string | undefined): Lang {
  if (stored === 'id' || stored === 'en') {
    return stored;
  }
  return browser !== undefined && browser.toLowerCase().startsWith('en') ? 'en' : 'id';
}

/** Mengganti `{nama}` di `template` dengan nilai `values`. */
export function fill(template: string, values: Record<string, string>): string {
  return template.replace(/\{([a-z]+)\}/g, (whole: string, name: string) => values[name] ?? whole);
}
