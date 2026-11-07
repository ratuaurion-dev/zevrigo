import type { Lang } from './i18n';

export const QUANTA_DECIMALS = 18;
export const QUANTA_PER_ZVRG = 1_000_000_000_000_000_000n; // 10^18
export const U128_MAX = (1n << 128n) - 1n; // 340282366920938463463374607431768211455n
export const MINING_SUPPLY_QUANTA = 100_000_000n * QUANTA_PER_ZVRG;
export const GENESIS_ALLOCATION_QUANTA = 10_000_000n * QUANTA_PER_ZVRG;

const QUANTA_REGEX = /^(0|[1-9][0-9]{0,38})$/;

/**
 * Strictly validates and parses a decimal quanta string into a BigInt.
 * Rejects leading zeros (except "0"), negative signs, decimals, whitespace,
 * non-digits, and values exceeding u128::MAX (2^128 - 1).
 */
export function parseQuanta(raw: string): bigint | null {
  if (typeof raw !== 'string' || !QUANTA_REGEX.test(raw)) {
    return null;
  }
  const value = BigInt(raw);
  if (value < 0n || value > U128_MAX) {
    return null;
  }
  return value;
}

/**
 * Formats an integer string of digits with language-aware thousands separators.
 * Indonesian ('id') uses '.' and English ('en') uses ','.
 */
export function formatDigitsWithThousands(digits: string, lang: Lang): string {
  const sep = lang === 'id' ? '.' : ',';
  const clean = digits.replace(/^0+(?=\d)/, '');
  const len = clean.length;
  if (len <= 3) return clean;

  const parts: string[] = [];
  let i = len;
  while (i > 0) {
    const start = i - 3 > 0 ? i - 3 : 0;
    parts.unshift(clean.slice(start, i));
    i = start;
  }
  return parts.join(sep);
}

/**
 * Formats a small non-coin integer (height, byte size, count) with thousands separators.
 * Does NOT use the browser number formatter.
 */
export function formatInt(value: number, lang: Lang): string {
  if (!Number.isFinite(value)) return '0';
  const isNegative = value < 0;
  const absStr = String(isNegative ? -Math.trunc(value) : Math.trunc(value));
  const formatted = formatDigitsWithThousands(absStr, lang);
  return isNegative ? `-${formatted}` : formatted;
}

/**
 * Formats a BigInt quanta amount with thousands separators + " quanta" suffix.
 */
export function formatQuantaExact(quanta: bigint | string, lang: Lang): string {
  const q = typeof quanta === 'bigint' ? quanta : parseQuanta(quanta);
  if (q === null) return '—';
  return `${formatDigitsWithThousands(q.toString(10), lang)} quanta`;
}

export interface FormatZvrgOptions {
  includeTicker?: boolean;
  maxDecimals?: number; // If specified (< 18), truncates using pure BigInt / string slicing
  signed?: 'plus' | 'minus' | 'none';
}

/**
 * Formats a quanta value (BigInt or validated decimal string) into ZVRG.
 * - Up to 18 decimal places, trailing zeros stripped.
 * - Thousands separator: '.' for Indonesian, ',' for English.
 * - Decimal separator: ',' for Indonesian, '.' for English.
 * - Never uses JavaScript Number or floating point math.
 */
export function formatZvrg(
  quantaInput: bigint | string,
  lang: Lang,
  options?: FormatZvrgOptions
): string {
  const q = typeof quantaInput === 'bigint' ? quantaInput : parseQuanta(quantaInput);
  if (q === null) {
    return '—';
  }

  const includeTicker = options?.includeTicker ?? true;
  const maxDecimals = options?.maxDecimals ?? QUANTA_DECIMALS;
  const decSep = lang === 'id' ? ',' : '.';

  const wholePart = q / QUANTA_PER_ZVRG;
  const fracPart = q % QUANTA_PER_ZVRG;

  const formattedWhole = formatDigitsWithThousands(wholePart.toString(10), lang);

  let fracStr = fracPart.toString(10).padStart(QUANTA_DECIMALS, '0');
  if (maxDecimals >= 0 && maxDecimals < QUANTA_DECIMALS) {
    fracStr = fracStr.slice(0, maxDecimals);
  }
  // Remove trailing zeros
  fracStr = fracStr.replace(/0+$/, '');

  // If q > 0 but truncation removed all non-zero digits, indicate "< 0.0001" or show at least first significant digit
  let numberStr: string;
  if (fracStr.length > 0) {
    numberStr = `${formattedWhole}${decSep}${fracStr}`;
  } else {
    numberStr = formattedWhole;
  }

  const prefix =
    options?.signed === 'plus' && q > 0n
      ? '+'
      : options?.signed === 'minus' && q > 0n
      ? '−'
      : '';

  return includeTicker ? `${prefix}${numberStr} ZVRG` : `${prefix}${numberStr}`;
}

/**
 * Compact ZVRG formatter using pure BigInt truncation (e.g., max 4 or 6 decimals)
 * while preserving exact non-zero visibility for tiny amounts.
 */
export function formatZvrgCompact(
  quantaInput: bigint | string,
  lang: Lang,
  maxDecimals = 4,
  includeTicker = true
): string {
  const q = typeof quantaInput === 'bigint' ? quantaInput : parseQuanta(quantaInput);
  if (q === null) return '—';
  if (q === 0n) return includeTicker ? '0 ZVRG' : '0';

  const fullFrac = (q % QUANTA_PER_ZVRG).toString(10).padStart(QUANTA_DECIMALS, '0');
  const whole = q / QUANTA_PER_ZVRG;
  const truncatedFrac = fullFrac.slice(0, maxDecimals).replace(/0+$/, '');

  if (whole === 0n && truncatedFrac.length === 0) {
    // Find first non-zero decimal position so we don't mislead with "0 ZVRG" for non-zero quanta
    const firstNonZero = fullFrac.search(/[1-9]/);
    const keep = firstNonZero >= 0 ? firstNonZero + 2 : maxDecimals;
    return formatZvrg(q, lang, {
      includeTicker,
      maxDecimals: keep > QUANTA_DECIMALS ? QUANTA_DECIMALS : keep,
    });
  }

  return formatZvrg(q, lang, { includeTicker, maxDecimals });
}

/**
 * Adds two quanta strings safely using BigInt.
 */
export function addQuanta(a: string, b: string): bigint | null {
  const qa = parseQuanta(a);
  const qb = parseQuanta(b);
  if (qa === null || qb === null) return null;
  const sum = qa + qb;
  return sum <= U128_MAX ? sum : null;
}

/**
 * Subtracts b from a safely using BigInt (clamped at 0n if b > a).
 */
export function subQuanta(a: string, b: string): bigint | null {
  const qa = parseQuanta(a);
  const qb = parseQuanta(b);
  if (qa === null || qb === null) return null;
  return qa >= qb ? qa - qb : 0n;
}

/**
 * Computes integer fee rate in quanta/byte using pure BigInt integer division.
 */
export function computeFeeRateQuantaPerByte(feeQuanta: string, sizeBytes: number): bigint | null {
  const fee = parseQuanta(feeQuanta);
  if (fee === null || !Number.isInteger(sizeBytes) || sizeBytes <= 0) return null;
  return fee / BigInt(sizeBytes);
}

/**
 * Computes basis points (0..10000, where 10000 = 100.00%) using pure BigInt math.
 */
export function computeBasisPoints(numerator: bigint | string, denominator: bigint | string): bigint {
  const num = typeof numerator === 'bigint' ? numerator : parseQuanta(numerator);
  const den = typeof denominator === 'bigint' ? denominator : parseQuanta(denominator);
  if (num === null || den === null || den <= 0n || num <= 0n) {
    return 0n;
  }
  const bps = (num * 10_000n) / den;
  return bps > 10_000n ? 10_000n : bps;
}

/**
 * Formats basis points (0n..10000n) as a percentage string (e.g. "42,15%" in ID or "42.15%" in EN).
 * For non-zero numerator that rounds to 0 bps, shows "< 0,01%" / "< 0.01%".
 */
export function formatBasisPointsPercent(
  numerator: bigint | string,
  denominator: bigint | string,
  lang: Lang
): string {
  const num = typeof numerator === 'bigint' ? numerator : parseQuanta(numerator);
  const den = typeof denominator === 'bigint' ? denominator : parseQuanta(denominator);
  const decSep = lang === 'id' ? ',' : '.';
  if (num === null || den === null || den <= 0n || num <= 0n) {
    return `0${decSep}00%`;
  }
  const bps = (num * 10_000n) / den;
  if (bps === 0n && num > 0n) {
    return `< 0${decSep}01%`;
  }
  const clamped = bps > 10_000n ? 10_000n : bps;
  const whole = clamped / 100n;
  const rem = (clamped % 100n).toString(10).padStart(2, '0');
  return `${whole.toString(10)}${decSep}${rem}%`;
}
