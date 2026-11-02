import { describe, expect, it } from 'vitest';
import { formatUtc } from './time';

describe('time', () => {
  it('formats unix seconds as UTC', () => {
    expect(formatUtc(0)).toBe('1970-01-01 00:00:00 UTC');
    expect(formatUtc(951_782_400)).toBe('2000-02-29 00:00:00 UTC');
    expect(formatUtc(1_700_000_000)).toBe('2023-11-14 22:13:20 UTC');
    expect(formatUtc(4_102_444_799)).toBe('2099-12-31 23:59:59 UTC');
  });

  it('rejects invalid seconds', () => {
    expect(formatUtc(-1)).toBe('—');
    expect(formatUtc(1.5)).toBe('—');
    expect(formatUtc(Number.MAX_SAFE_INTEGER + 1)).toBe('—');
  });
});
