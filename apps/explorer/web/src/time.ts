// Waktu blok (detik Unix) sebagai UTC, dihitung dengan bigint (algoritma tanggal sipil Howard Hinnant), tanpa zona
// waktu browser.

function pad(value: bigint, width: number): string {
  return value.toString().padStart(width, '0');
}

/** Detik Unix ke `YYYY-MM-DD HH:MM:SS UTC`; `—` untuk nilai yang bukan bilangan bulat tak negatif. */
export function formatUtc(seconds: number): string {
  if (!Number.isSafeInteger(seconds) || seconds < 0) {
    return '—';
  }
  const total = BigInt(seconds);
  const rest = total % 86_400n;
  const z = total / 86_400n + 719_468n;
  const era = z / 146_097n;
  const doe = z - era * 146_097n;
  const yoe = (doe - doe / 1_460n + doe / 36_524n - doe / 146_096n) / 365n;
  const doy = doe - (365n * yoe + yoe / 4n - yoe / 100n);
  const mp = (5n * doy + 2n) / 153n;
  const day = doy - (153n * mp + 2n) / 5n + 1n;
  const month = mp < 10n ? mp + 3n : mp - 9n;
  const year = yoe + era * 400n + (month <= 2n ? 1n : 0n);
  const clock = `${pad(rest / 3_600n, 2)}:${pad((rest % 3_600n) / 60n, 2)}:${pad(rest % 60n, 2)}`;
  return `${pad(year, 4)}-${pad(month, 2)}-${pad(day, 2)} ${clock} UTC`;
}
