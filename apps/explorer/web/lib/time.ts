import type { Lang } from './i18n';

const SECONDS_PER_DAY = 86_400n;

/**
 * Floor division for BigInt (handles negative numbers toward -Infinity).
 */
function floorDiv(a: bigint, b: bigint): bigint {
  const q = a / b;
  const r = a % b;
  if (r !== 0n && ((r < 0n) !== (b < 0n))) {
    return q - 1n;
  }
  return q;
}

/**
 * Positive modulo for BigInt.
 */
function posMod(a: bigint, b: bigint): bigint {
  const r = a % b;
  return r < 0n ? r + b : r;
}

/**
 * Converts days since 1970-01-01 (Unix epoch) to [year, month, day] in the proleptic Gregorian calendar.
 * Uses Howard Hinnant's civil_from_days algorithm with pure BigInt integer arithmetic.
 * Zero dependence on Date or browser timezone.
 */
export function civilFromDays(daysSinceEpoch: bigint): {
  year: number;
  month: number;
  day: number;
} {
  const z = daysSinceEpoch + 719_468n;
  const era = floorDiv(z, 146_097n);
  const doe = z - era * 146_097n; // [0, 146096]
  const yoe = (doe - doe / 1_460n + doe / 36_524n - doe / 146_096n) / 365n; // [0, 399]
  const y = yoe + era * 400n;
  const doy = doe - (365n * yoe + yoe / 4n - yoe / 100n); // [0, 365]
  const mp = (5n * doy + 2n) / 153n; // [0, 11]
  const d = doy - (153n * mp + 2n) / 5n + 1n; // [1, 31]
  const m = mp < 10n ? mp + 3n : mp - 9n; // [1, 12]
  const year = y + (m <= 2n ? 1n : 0n);

  return {
    year: Number(year),
    month: Number(m),
    day: Number(d),
  };
}

function pad2(n: number): string {
  return n < 10 ? `0${n}` : `${n}`;
}

function pad4(n: number): string {
  if (n < 0) {
    return `-${String(-n).padStart(4, '0')}`;
  }
  return String(n).padStart(4, '0');
}

/**
 * Formats a Unix timestamp (in integer seconds) as `YYYY-MM-DD HH:MM:SS UTC`
 * using pure integer math (civil-from-days algorithm).
 */
export function formatUtcTimestamp(unixSeconds: number): string {
  if (!Number.isFinite(unixSeconds)) {
    return '—';
  }
  const sec = BigInt(Math.trunc(unixSeconds));
  const days = floorDiv(sec, SECONDS_PER_DAY);
  const secOfDay = Number(posMod(sec, SECONDS_PER_DAY));

  const hours = Math.trunc(secOfDay / 3600);
  const remAfterHours = secOfDay - hours * 3600;
  const minutes = Math.trunc(remAfterHours / 60);
  const seconds = remAfterHours - minutes * 60;

  const { year, month, day } = civilFromDays(days);
  return `${pad4(year)}-${pad2(month)}-${pad2(day)} ${pad2(hours)}:${pad2(minutes)}:${pad2(seconds)} UTC`;
}

/**
 * Computes a bilingual relative time label from integer Unix seconds.
 */
export function formatRelativeTime(
  unixSeconds: number,
  nowUnixSeconds: number,
  lang: Lang
): string {
  const target = Math.trunc(unixSeconds);
  const now = Math.trunc(nowUnixSeconds);
  const diff = now - target;

  if (diff <= 2) {
    return lang === 'id' ? 'baru saja' : 'just now';
  }
  if (diff < 60) {
    return lang === 'id' ? `${diff} detik lalu` : `${diff}s ago`;
  }
  const minutes = Math.trunc(diff / 60);
  if (minutes < 60) {
    if (lang === 'id') {
      return `${minutes} menit lalu`;
    }
    return minutes === 1 ? '1 minute ago' : `${minutes} minutes ago`;
  }
  const hours = Math.trunc(minutes / 60);
  if (hours < 24) {
    if (lang === 'id') {
      return `${hours} jam lalu`;
    }
    return hours === 1 ? '1 hour ago' : `${hours} hours ago`;
  }
  const days = Math.trunc(hours / 24);
  if (lang === 'id') {
    return `${days} hari lalu`;
  }
  return days === 1 ? '1 day ago' : `${days} days ago`;
}
