import { createHash } from 'node:crypto';
import { describe, expect, test } from 'vitest';
import { WORDLIST, checkPhrase, quizPositions } from './bip39';

describe('bip39', () => {
  test('daftar kata sama dengan english.txt MOD-09', () => {
    expect(WORDLIST).toHaveLength(2048);
    expect(new Set(WORDLIST).size).toBe(2048);
    const sha = createHash('sha256').update(`${WORDLIST.join('\n')}\n`).digest('hex');
    expect(sha).toBe('2f5eed53a4727b4bf8880d8f3f199efc90e58503646d9ff8eff3a2ed3b24dbda');
  });

  test('memeriksa jumlah dan ejaan kata frasa', () => {
    const w1 = `${'abandon '.repeat(23)}art`;
    expect(checkPhrase(`  ${w1.toUpperCase().replace(/ /g, '\n  ')} `)).toEqual({
      normalized: w1,
      count: 24,
      countOk: true,
      unknown: [],
    });
    expect(checkPhrase('legal winner thank year wave sausage worth useful legal winner thank yellow').countOk).toBe(
      true,
    );
    const typo = checkPhrase('abandon abandn art');
    expect(typo.count).toBe(3);
    expect(typo.countOk).toBe(false);
    expect(typo.unknown).toEqual([{ position: 2, word: 'abandn' }]);
  });

  test('memilih tiga nomor kata berbeda untuk diperiksa ulang', () => {
    const sequence = [5, 5, 23, 0];
    let i = 0;
    expect(quizPositions(24, () => sequence[i++] ?? 0)).toEqual([1, 6, 24]);
    for (let run = 0; run < 50; run++) {
      const picked = quizPositions(24);
      expect(new Set(picked).size).toBe(3);
      expect(picked.every((p) => p >= 1 && p <= 24)).toBe(true);
      expect([...picked].sort((a, b) => a - b)).toEqual(picked);
    }
  });
});
