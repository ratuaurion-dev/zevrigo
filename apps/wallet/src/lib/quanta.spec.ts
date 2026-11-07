import { describe, expect, test } from 'vitest';
import { canonicalZvrg, checkSendAmount, formatQuanta, formatZvrg, parseZvrg, toQuanta } from './quanta';

const ZVRG = 1_000_000_000_000_000_000n;

describe('quanta', () => {
  test('mengurai ZVRG utuh dan pecahan tepat ke quanta', () => {
    expect(parseZvrg('1')).toEqual({ ok: true, quanta: ZVRG });
    expect(parseZvrg(' 12.5 ')).toEqual({ ok: true, quanta: 12n * ZVRG + ZVRG / 2n });
    expect(parseZvrg('0,25')).toEqual({ ok: true, quanta: ZVRG / 4n });
    expect(parseZvrg('0.000000000000000001')).toEqual({ ok: true, quanta: 1n });
    expect(parseZvrg('100000000')).toEqual({ ok: true, quanta: 100_000_000n * ZVRG });
  });

  test('menolak masukan yang tidak sah', () => {
    expect(parseZvrg('')).toEqual({ ok: false, error: 'empty' });
    for (const text of ['1.2.3', '-1', '+1', '1e3', '1 000', '1,000.5', '.5', 'abc']) {
      expect(parseZvrg(text)).toEqual({ ok: false, error: 'format' });
    }
    expect(parseZvrg('0.0000000000000000001')).toEqual({ ok: false, error: 'precision' });
    expect(parseZvrg('0.000')).toEqual({ ok: false, error: 'zero' });
  });

  test('jumlah kirim tidak boleh melebihi saldo yang bisa dipakai', () => {
    expect(checkSendAmount('2', 2n * ZVRG)).toEqual({ ok: true, quanta: 2n * ZVRG });
    expect(checkSendAmount('2.000000000000000001', 2n * ZVRG)).toEqual({ ok: false, error: 'insufficient' });
    expect(checkSendAmount('x', 2n * ZVRG)).toEqual({ ok: false, error: 'format' });
  });

  test('menulis ZVRG kanonis dan tampilan per bahasa', () => {
    const base = 43_959_096_939_479_752_879n;
    expect(canonicalZvrg(base)).toBe('43.959096939479752879');
    expect(canonicalZvrg(2n * ZVRG)).toBe('2');
    expect(canonicalZvrg(-ZVRG / 2n)).toBe('-0.5');
    expect(canonicalZvrg(0n)).toBe('0');
    const big = 1_234_567n * ZVRG + ZVRG / 2n;
    expect(formatZvrg(big, 'id')).toEqual({ text: '1.234.567,5', cut: false });
    expect(formatZvrg(big, 'en')).toEqual({ text: '1,234,567.5', cut: false });
    expect(formatZvrg(base, 'id', 4)).toEqual({ text: '43,959', cut: true });
    expect(formatZvrg(2n * ZVRG, 'en', 4)).toEqual({ text: '2', cut: false });
    expect(formatZvrg(-1n, 'en', 4)).toEqual({ text: '-0', cut: true });
  });

  test('membaca teks quanta dari wallet hanya dalam bentuk kanonis', () => {
    expect(toQuanta('0')).toBe(0n);
    expect(toQuanta('43959096939479752879')).toBe(43_959_096_939_479_752_879n);
    for (const text of ['', '01', '1.5', '-1', ' 1']) {
      expect(() => toQuanta(text)).toThrow();
    }
    expect(formatQuanta(4_361_021_521_773_785n, 'id')).toBe('4.361.021.521.773.785');
    expect(formatQuanta(-1000n, 'en')).toBe('-1,000');
  });
});
