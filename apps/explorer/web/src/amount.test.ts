import { describe, expect, it } from 'vitest';
import { formatHsis, groupDigits, parseQuanta } from './amount';

const HSIS = 1_000_000_000_000_000_000n;

describe('amount', () => {
  it('parses canonical quanta only', () => {
    expect(parseQuanta('0')).toBe(0n);
    expect(parseQuanta('4380000000000000000000000')).toBe(4_380_000n * HSIS);
    expect(parseQuanta('340282366920938463463374607431768211455')).toBe((1n << 128n) - 1n);
    for (const bad of ['', '01', '-1', '1.5', '1e3', ' 1', '0x10', '340282366920938463463374607431768211456']) {
      expect(parseQuanta(bad)).toBeNull();
    }
  });

  it('formats HSIS in Indonesian', () => {
    expect(formatHsis(0n, 'id')).toBe('0 HSIS');
    expect(formatHsis(1n, 'id')).toBe('0,000000000000000001 HSIS');
    expect(formatHsis(4_380_000n * HSIS, 'id')).toBe('4.380.000 HSIS');
    expect(formatHsis(5_000_000_000_000_000n, 'id')).toBe('0,005 HSIS');
    expect(formatHsis(4_380_000n * HSIS - 1_000n, 'id')).toBe('4.379.999,999999999999999 HSIS');
  });

  it('formats HSIS in English', () => {
    expect(formatHsis(4_380_000n * HSIS, 'en')).toBe('4,380,000 HSIS');
    expect(formatHsis(5_000_000_000_000_000n, 'en')).toBe('0.005 HSIS');
    expect(formatHsis((1n << 128n) - 1n, 'en')).toBe('340,282,366,920,938,463,463.374607431768211455 HSIS');
  });

  it('groups digits', () => {
    expect(groupDigits('1', '.')).toBe('1');
    expect(groupDigits('123', '.')).toBe('123');
    expect(groupDigits('1234', '.')).toBe('1.234');
    expect(groupDigits('1234567', ',')).toBe('1,234,567');
  });
});
