// Jumlah koin (Spesifikasi Explorer Web v0.1 Bagian 4): string desimal quanta dari API -> bigint -> teks HSIS.
// Tidak pernah memakai tipe number untuk jumlah koin (larangan floating point AGENTS.md).
import type { Lang } from './i18n';

const QUANTA_PER_HSIS = 1_000_000_000_000_000_000n;
const DECIMALS = 18;
const MAX_QUANTA = (1n << 128n) - 1n;

/** quanta (desimal kanonis, paling besar u128) ke bigint; null jika tidak sah. */
export function parseQuanta(text: string): bigint | null {
  if (!/^(0|[1-9][0-9]{0,38})$/.test(text)) {
    return null;
  }
  const value = BigInt(text);
  return value <= MAX_QUANTA ? value : null;
}

/** Menyisipkan pemisah ribuan ke deret angka desimal. */
export function groupDigits(digits: string, separator: string): string {
  let out = '';
  for (let i = 0; i < digits.length; i += 1) {
    if (i > 0 && (digits.length - i) % 3 === 0) {
      out += separator;
    }
    out += digits.charAt(i);
  }
  return out;
}

/** Pemisah ribuan dan desimal per bahasa. */
export function separators(lang: Lang): { thousands: string; decimal: string } {
  return lang === 'en' ? { thousands: ',', decimal: '.' } : { thousands: '.', decimal: ',' };
}

/** quanta ke teks HSIS: 18 desimal, nol di belakang dibuang, pemisah sesuai bahasa. */
export function formatHsis(quanta: bigint, lang: Lang): string {
  const { thousands, decimal } = separators(lang);
  const whole = groupDigits((quanta / QUANTA_PER_HSIS).toString(), thousands);
  const fraction = (quanta % QUANTA_PER_HSIS).toString().padStart(DECIMALS, '0').replace(/0+$/, '');
  return `${fraction === '' ? whole : `${whole}${decimal}${fraction}`} HSIS`;
}
