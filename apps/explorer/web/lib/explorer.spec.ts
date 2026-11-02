import { describe, expect, it } from 'vitest';
import {
  computeBasisPoints,
  computeFeeRateQuantaPerByte,
  formatBasisPointsPercent,
  formatHsis,
  formatInt,
  parseQuanta,
  U128_MAX,
} from './amount';
import { formatRelativeTime, formatUtcTimestamp } from './time';
import {
  classifySearchQuery,
  isValidAddress,
  isValidHash,
  isValidHeight,
  isValidPage,
  routes,
} from './route';
import { en, id, translate } from './i18n';
import { fetchAddress, fetchAllocation, fetchBlocks, fetchTx, requestJson } from './api';
import { API_BASE_PATH, USE_MOCK } from './config';

describe('amount parse and format (BigInt only)', () => {
  it('parses valid quanta strings up to u128 max and rejects invalid inputs', () => {
    expect(parseQuanta('0')).toBe(0n);
    expect(parseQuanta('1')).toBe(1n);
    expect(parseQuanta('4380000000000000000000000')).toBe(
      4_380_000_000_000_000_000_000_000n
    );
    expect(parseQuanta(U128_MAX.toString(10))).toBe(U128_MAX);

    // Rejects invalid inputs
    expect(parseQuanta('')).toBeNull();
    expect(parseQuanta('00')).toBeNull();
    expect(parseQuanta('0123')).toBeNull();
    expect(parseQuanta('-1')).toBeNull();
    expect(parseQuanta('1.5')).toBeNull();
    expect(parseQuanta('1000n')).toBeNull();
    expect(parseQuanta(' 100 ')).toBeNull();
    expect(parseQuanta((U128_MAX + 1n).toString(10))).toBeNull();
  });

  it('formats HSIS amounts in Indonesian (ID) and English (EN)', () => {
    // 0
    expect(formatHsis('0', 'id')).toBe('0 HSIS');
    expect(formatHsis('0', 'en')).toBe('0 HSIS');

    // 1 quanta = 0.000000000000000001 HSIS
    expect(formatHsis('1', 'id')).toBe('0,000000000000000001 HSIS');
    expect(formatHsis('1', 'en')).toBe('0.000000000000000001 HSIS');

    // 0.005 HSIS = 5 * 10^15 quanta
    expect(formatHsis('5000000000000000', 'id')).toBe('0,005 HSIS');
    expect(formatHsis('5000000000000000', 'en')).toBe('0.005 HSIS');

    // 4,380,000 HSIS (genesis allocation)
    expect(formatHsis('4380000000000000000000000', 'id')).toBe(
      '4.380.000 HSIS'
    );
    expect(formatHsis('4380000000000000000000000', 'en')).toBe(
      '4,380,000 HSIS'
    );

    // u128 max (340282366920938463463374607431768211455 quanta)
    expect(formatHsis(U128_MAX, 'en')).toBe(
      '340,282,366,920,938,463,463.374607431768211455 HSIS'
    );
    expect(formatHsis(U128_MAX, 'id')).toBe(
      '340.282.366.920.938.463.463,374607431768211455 HSIS'
    );
  });

  it('computes basis points and fee rates with integer math', () => {
    expect(
      computeBasisPoints(
        '2190000000000000000000000',
        '4380000000000000000000000'
      )
    ).toBe(5000n);
    expect(
      formatBasisPointsPercent(
        '2190000000000000000000000',
        '4380000000000000000000000',
        'id'
      )
    ).toBe('50,00%');
    expect(
      formatBasisPointsPercent(
        '2190000000000000000000000',
        '4380000000000000000000000',
        'en'
      )
    ).toBe('50.00%');
    expect(computeFeeRateQuantaPerByte('1000', 385)).toBe(2n);
    expect(formatInt(100000, 'id')).toBe('100.000');
    expect(formatInt(100000, 'en')).toBe('100,000');
  });
});

describe('UTC and relative time formatting (civil-from-days integer math)', () => {
  it('formats epoch, leap day 2000-02-29, and 2099-12-31 23:59:59 UTC accurately', () => {
    // 1970-01-01 00:00:00 UTC
    expect(formatUtcTimestamp(0)).toBe('1970-01-01 00:00:00 UTC');

    // 2000-02-29 12:34:56 UTC -> 951827696
    expect(formatUtcTimestamp(951_827_696)).toBe('2000-02-29 12:34:56 UTC');

    // 2099-12-31 23:59:59 UTC -> 4102444799
    expect(formatUtcTimestamp(4_102_444_799)).toBe('2099-12-31 23:59:59 UTC');
  });

  it('formats relative labels in ID and EN', () => {
    expect(formatRelativeTime(1000, 1180, 'id')).toBe('3 menit lalu');
    expect(formatRelativeTime(1000, 1180, 'en')).toBe('3 minutes ago');
  });
});

describe('validators and href builders', () => {
  it('validates height, hash, bech32m address, and page', () => {
    expect(isValidHeight('0')).toBe(true);
    expect(isValidHeight('120')).toBe(true);
    expect(isValidHeight('01')).toBe(false);
    expect(isValidHeight('-5')).toBe(false);

    const validHash = 'a'.repeat(64);
    expect(isValidHash(validHash)).toBe(true);
    expect(isValidHash('A'.repeat(64))).toBe(false);
    expect(isValidHash('a'.repeat(63))).toBe(false);

    expect(
      isValidAddress('rhsis1qpzry9x8gf2tvdw0s3jn54khce6mua7lqpzry9')
    ).toBe(true);
    expect(
      isValidAddress('hsis1qpzry9x8gf2tvdw0s3jn54khce6mua7lqpzry9')
    ).toBe(true);
    expect(
      isValidAddress('thsis1qpzry9x8gf2tvdw0s3jn54khce6mua7lqpzry9')
    ).toBe(true);
    expect(isValidAddress('btc1qpzry9x8gf2tvdw0s3jn54khce6mua7l')).toBe(
      false
    );
    expect(isValidAddress('rhsis1invalidcharb')).toBe(false);

    expect(isValidPage('0')).toBe(true);
    expect(isValidPage('25')).toBe(true);
    expect(isValidPage('01')).toBe(false);

    expect(classifySearchQuery(' 120 ')).toEqual({
      kind: 'height',
      normalized: '120',
    });
  });

  it('builds query-param static export routes correctly', () => {
    expect(routes.home()).toBe('/');
    expect(routes.blocks()).toBe('/blocks/');
    expect(routes.blocks(100)).toBe('/blocks/?before=100');
    expect(routes.block(120)).toBe('/block/?id=120');
    expect(routes.tx('abc')).toBe('/tx/?id=abc');
    expect(routes.address('rhsis1qpzry9', 2)).toBe(
      '/address/?a=rhsis1qpzry9&page=2'
    );
    expect(routes.allocation()).toBe('/allocation/');
    expect(routes.allocation(1)).toBe('/allocation/?page=1');
  });
});

describe('i18n dictionaries', () => {
  it('has identical keys and identical {placeholder} tokens in id and en', () => {
    const idKeys = Object.keys(id).sort();
    const enKeys = Object.keys(en).sort();
    expect(idKeys).toEqual(enKeys);

    const extractPlaceholders = (s: string) =>
      Array.from(s.matchAll(/\{([a-zA-Z0-9_]+)\}/g))
        .map((m) => m[1])
        .sort();

    for (const key of idKeys as (keyof typeof id)[]) {
      expect(extractPlaceholders(id[key])).toEqual(
        extractPlaceholders(en[key])
      );
    }

    expect(translate('id', 'bannerSyncingText', { indexed: 50, node: 120 })).toBe(
      'Menyinkronkan: 50 dari 120 blok'
    );
    expect(translate('en', 'bannerSyncingText', { indexed: 50, node: 120 })).toBe(
      'Syncing: 50 of 120 blocks'
    );
  });
});

describe('API client error mapping', () => {
  it('maps 404 with in_mempool: true', async () => {
    const mockFetch: typeof fetch = async () =>
      new Response(JSON.stringify({ error: 'not_found', in_mempool: true }), {
        status: 404,
        headers: { 'Content-Type': 'application/json' },
      });

    const res = await requestJson('/tx/deadbeef', mockFetch);
    expect(res).toEqual({
      ok: false,
      failure: {
        kind: 'http',
        status: 404,
        error: 'not_found',
        inMempool: true,
      },
    });
  });

  it('maps 503 not_ready', async () => {
    const mockFetch: typeof fetch = async () =>
      new Response(JSON.stringify({ error: 'not_ready' }), {
        status: 503,
        headers: { 'Content-Type': 'application/json' },
      });

    const res = await requestJson('/blocks', mockFetch);
    expect(res).toEqual({
      ok: false,
      failure: {
        kind: 'http',
        status: 503,
        error: 'not_ready',
      },
    });
  });

  it('maps network errors without throwing', async () => {
    const mockFetch: typeof fetch = async () => {
      throw new Error('Connection refused');
    };

    const res = await requestJson('/status', mockFetch);
    expect(res).toEqual({
      ok: false,
      failure: {
        kind: 'network',
        message: 'Connection refused',
      },
    });
  });

  it('maps malformed JSON to format failure', async () => {
    const mockFetch: typeof fetch = async () =>
      new Response('not-valid-json{{{', {
        status: 200,
      });

    const res = await requestJson('/status', mockFetch);
    expect(res.ok).toBe(false);
    if (!res.ok) {
      expect(res.failure.kind).toBe('format');
    }
  });
});

describe('API paths and mock mode', () => {
  it('builds same-origin API paths with encoded keys', async () => {
    const seen: string[] = [];
    const fetchFn: typeof fetch = async (input) => {
      seen.push(String(input));
      return new Response('{}', { status: 200 });
    };
    const txid = 'ab'.repeat(32);
    await fetchBlocks(undefined, fetchFn);
    await fetchBlocks(5, fetchFn);
    await fetchTx(txid, fetchFn);
    await fetchAddress('rhsis1qqq', 2, fetchFn);
    await fetchAllocation(0, fetchFn);
    expect(API_BASE_PATH).toBe('/api/v1');
    expect(seen).toEqual([
      '/api/v1/blocks',
      '/api/v1/blocks?before=5',
      `/api/v1/tx/${txid}`,
      '/api/v1/address/rhsis1qqq?page=2',
      '/api/v1/allocation?page=0',
    ]);
  });

  it('keeps mock data off unless NEXT_PUBLIC_USE_MOCK=1', () => {
    expect(process.env.NEXT_PUBLIC_USE_MOCK).toBeUndefined();
    expect(USE_MOCK).toBe(false);
  });
});
