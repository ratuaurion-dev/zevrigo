/**
 * Jumlah koin di antarmuka wallet (MOD-15): 1 ZVRG = 10^18 quanta.
 *
 * Aturan wajib: jumlah koin tidak pernah memakai `number` (floating point). Di IPC jumlah berupa teks angka quanta,
 * di antarmuka berupa `bigint`. Teks masukan diurai dengan memecah string, tanpa bilangan pecahan atau pembulatan.
 */

export const QUANTA_DECIMALS = 18;
export const QUANTA_PER_ZVRG = 1_000_000_000_000_000_000n;

export type AmountError = 'empty' | 'format' | 'precision' | 'zero' | 'insufficient';

export type Lang = 'id' | 'en';

/** Teks angka quanta dari Rust ("0", "123", ...) menjadi bigint. Teks lain ditolak. */
export function toQuanta(text: string): bigint {
  if (!/^(0|[1-9][0-9]*)$/.test(text)) {
    throw new Error(`jumlah quanta tidak sah: "${text}"`);
  }
  return BigInt(text);
}

/**
 * Teks ZVRG dari pengguna ("1", "12.5", "0,25") menjadi quanta. Pemisah desimal titik atau koma (satu saja),
 * tanpa pemisah ribuan, tanpa tanda, maksimal 18 angka di belakang koma.
 */
export function parseZvrg(input: string): { ok: true; quanta: bigint } | { ok: false; error: AmountError } {
  const text = input.trim();
  if (text === '') {
    return { ok: false, error: 'empty' };
  }
  const match = /^([0-9]+)(?:[.,]([0-9]+))?$/.exec(text);
  if (!match) {
    return { ok: false, error: 'format' };
  }
  const whole = match[1] ?? '0';
  const fraction = match[2] ?? '';
  if (fraction.length > QUANTA_DECIMALS) {
    return { ok: false, error: 'precision' };
  }
  const quanta = BigInt(whole) * QUANTA_PER_ZVRG + BigInt(fraction.padEnd(QUANTA_DECIMALS, '0') || '0');
  if (quanta === 0n) {
    return { ok: false, error: 'zero' };
  }
  return { ok: true, quanta };
}

/** Jumlah kirim: sah dan tidak melebihi saldo yang bisa dipakai (biaya diperiksa wallet saat menyusun transaksi). */
export function checkSendAmount(
  input: string,
  spendable: bigint,
): { ok: true; quanta: bigint } | { ok: false; error: AmountError } {
  const parsed = parseZvrg(input);
  if (parsed.ok && parsed.quanta > spendable) {
    return { ok: false, error: 'insufficient' };
  }
  return parsed;
}

/** Teks ZVRG kanonis untuk URI dan salinan: titik desimal, tanpa pemisah ribuan, tanpa nol di belakang. */
export function canonicalZvrg(quanta: bigint): string {
  const negative = quanta < 0n;
  const abs = negative ? -quanta : quanta;
  const whole = (abs / QUANTA_PER_ZVRG).toString();
  const fraction = (abs % QUANTA_PER_ZVRG).toString().padStart(QUANTA_DECIMALS, '0').replace(/0+$/, '');
  return `${negative ? '-' : ''}${whole}${fraction ? `.${fraction}` : ''}`;
}

function groupWhole(whole: string, separator: string): string {
  return whole.replace(/\B(?=(\d{3})+(?!\d))/g, separator);
}

/**
 * ZVRG untuk dibaca: pemisah ribuan dan desimal sesuai bahasa (id: 1.234,5; en: 1,234.5). Dengan `maxFraction`,
 * angka di belakang koma dipotong (tidak dibulatkan) dan `cut` bernilai true bila ada angka yang dipotong.
 */
export function formatZvrg(quanta: bigint, lang: Lang, maxFraction?: number): { text: string; cut: boolean } {
  const negative = quanta < 0n;
  const abs = negative ? -quanta : quanta;
  const whole = (abs / QUANTA_PER_ZVRG).toString();
  let fraction = (abs % QUANTA_PER_ZVRG).toString().padStart(QUANTA_DECIMALS, '0').replace(/0+$/, '');
  let cut = false;
  if (maxFraction !== undefined && fraction.length > maxFraction) {
    cut = /[1-9]/.test(fraction.slice(maxFraction));
    fraction = fraction.slice(0, maxFraction).replace(/0+$/, '');
  }
  const [thousands, decimal] = lang === 'id' ? ['.', ','] : [',', '.'];
  const text = `${negative ? '-' : ''}${groupWhole(whole, thousands)}${fraction ? `${decimal}${fraction}` : ''}`;
  return { text, cut };
}

/** Quanta untuk dibaca, dengan pemisah ribuan sesuai bahasa. */
export function formatQuanta(quanta: bigint, lang: Lang): string {
  const negative = quanta < 0n;
  const abs = negative ? -quanta : quanta;
  return `${negative ? '-' : ''}${groupWhole(abs.toString(), lang === 'id' ? '.' : ',')}`;
}
