/**
 * Kode QR untuk alamat dan URI pembayaran `zevrigo:<alamat>?amount=<ZVRG>`: matriks dari pustaka `qrcode`, digambar
 * sebagai satu path SVG (tanpa gambar dari luar).
 */

import { create as createQr } from 'qrcode';

export interface QrSvg {
  viewBox: number;
  path: string;
  modules: number;
}

export function paymentUri(address: string, amountZvrg?: string): string {
  return amountZvrg ? `zevrigo:${address}?amount=${amountZvrg}` : `zevrigo:${address}`;
}

export function qrSvg(text: string, margin = 2): QrSvg {
  const qr = createQr(text, { errorCorrectionLevel: 'M' });
  const size = qr.modules.size;
  const parts: string[] = [];
  for (let row = 0; row < size; row++) {
    for (let col = 0; col < size; col++) {
      if (qr.modules.get(row, col)) {
        parts.push(`M${col + margin} ${row + margin}h1v1h-1z`);
      }
    }
  }
  return { viewBox: size + margin * 2, path: parts.join(''), modules: size };
}
