import { id, type Dictionary, type DictionaryKey } from './id';
import { en } from './en';

export type Lang = 'id' | 'en';
export const LANG_STORAGE_KEY = 'hsis-explorer-lang';

export const dictionaries: Record<Lang, Dictionary> = {
  id,
  en,
};

/**
 * Interpolates `{placeholder}` tokens in a dictionary string.
 */
export function translate(
  lang: Lang,
  key: DictionaryKey,
  params?: Record<string, string | number>
): string {
  const dict = dictionaries[lang] ?? dictionaries.id;
  const template = dict[key] ?? id[key] ?? key;
  if (!params) return template;

  return template.replace(/\{([a-zA-Z0-9_]+)\}/g, (_, token: string) => {
    const val = params[token];
    return val !== undefined ? String(val) : `{${token}}`;
  });
}

/**
 * Detects initial language on the client:
 * 1. Saved value in localStorage key `hsis-explorer-lang` ('id' | 'en')
 * 2. `navigator.language` starting with 'en' -> 'en'
 * 3. Default -> 'id' (Bahasa Indonesia)
 */
export function detectInitialLang(): Lang {
  if (typeof window === 'undefined') {
    return 'id';
  }
  try {
    const saved = window.localStorage.getItem(LANG_STORAGE_KEY);
    if (saved === 'id' || saved === 'en') {
      return saved;
    }
  } catch {
    // Ignore storage restriction errors
  }
  if (typeof navigator !== 'undefined' && typeof navigator.language === 'string') {
    if (navigator.language.toLowerCase().startsWith('en')) {
      return 'en';
    }
  }
  return 'id';
}

export { id, en, type Dictionary, type DictionaryKey };
