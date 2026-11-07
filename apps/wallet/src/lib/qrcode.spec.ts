import { describe, expect, test } from 'vitest';
import { paymentUri, qrSvg } from './qrcode';

const W10 = 'tzvrg1qtfhjsfflxh307x4skzrt3jrlwvg4jes32ljmut9fsxyrd939mseq4f4dkt';

describe('kode QR', () => {
  test('menyusun URI pembayaran dan gambar QR yang tetap', () => {
    expect(paymentUri(W10)).toBe(`zevrigo:${W10}`);
    expect(paymentUri(W10, '0.25')).toBe(`zevrigo:${W10}?amount=0.25`);
    const svg = qrSvg(paymentUri(W10, '0.25'));
    expect(svg.modules).toBeGreaterThanOrEqual(21);
    expect((svg.modules - 17) % 4).toBe(0);
    expect(svg.viewBox).toBe(svg.modules + 4);
    expect(svg.path.startsWith('M2 2h1v1h-1z')).toBe(true);
    expect(qrSvg(paymentUri(W10, '0.25'))).toEqual(svg);
    expect(qrSvg(paymentUri(W10)).path).not.toBe(svg.path);
  });
});
