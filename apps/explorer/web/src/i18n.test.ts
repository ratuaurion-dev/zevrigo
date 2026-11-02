import { describe, expect, it } from 'vitest';
import { detectLanguage, dict, fill } from './i18n';
import { en } from './i18n/en';
import { id } from './i18n/id';

function placeholders(text: string): string[] {
  return (text.match(/\{[a-z]+\}/g) ?? []).slice().sort();
}

describe('i18n', () => {
  it('dictionaries have the same keys and no empty text', () => {
    expect(Object.keys(en).sort()).toEqual(Object.keys(id).sort());
    for (const value of [...Object.values(id), ...Object.values(en)]) {
      expect(value.trim()).not.toBe('');
    }
  });

  it('placeholders match between languages', () => {
    for (const key of Object.keys(id) as (keyof typeof id)[]) {
      expect(placeholders(en[key]), key).toEqual(placeholders(id[key]));
    }
  });

  it('detects language', () => {
    expect(detectLanguage(null, 'en-US')).toBe('en');
    expect(detectLanguage(null, 'EN')).toBe('en');
    expect(detectLanguage(null, 'id-ID')).toBe('id');
    expect(detectLanguage(null, 'de-DE')).toBe('id');
    expect(detectLanguage(null, undefined)).toBe('id');
    expect(detectLanguage('en', 'id-ID')).toBe('en');
    expect(detectLanguage('id', 'en-US')).toBe('id');
    expect(detectLanguage('fr', 'en-US')).toBe('en');
    expect(dict('en')).toBe(en);
    expect(dict('id')).toBe(id);
  });

  it('fills placeholders', () => {
    expect(fill('{n} konfirmasi', { n: '3' })).toBe('3 konfirmasi');
    expect(fill('Blok {h} {h}', { h: '7' })).toBe('Blok 7 7');
    expect(fill('{x} tetap', {})).toBe('{x} tetap');
  });
});
