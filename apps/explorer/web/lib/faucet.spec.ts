import { describe, expect, it } from 'vitest';
import type { FetchImpl } from './api';
import { FAUCET_URL, USE_MOCK } from './config';
import {
  faucetErrorKey,
  fetchFaucetStatus,
  isRegtestAddress,
  normalizeAddress,
  requestFaucet,
} from './faucet';
import { en, id } from './i18n';

/** Alamat regtest sungguhan jaringan demo (kantong Faucet treasury dan layanan faucet). */
const POCKET = 'rzvrg1qyxnvc28tl47kpamrex8z9sfws78ude9leyks6wjy32qd60y83y6s8cfl7y';
const FAUCET = 'rzvrg1qgjknjtac9n0jv3gvy7ffr76l6pld4kms3nkwzf8kk3204yfqt2qs4jxn6k';

type Call = { url: string; init: RequestInit | undefined };

function fakeFetch(status: number, body: unknown, calls: Call[] = []): FetchImpl {
  return (async (url: RequestInfo | URL, init?: RequestInit) => {
    calls.push({ url: String(url), init });
    return new Response(typeof body === 'string' ? body : JSON.stringify(body), {
      status,
      headers: { 'Content-Type': 'application/json' },
    });
  }) as unknown as FetchImpl;
}

describe('faucet address check (bech32m regtest jenis 0)', () => {
  it('accepts real regtest addresses, trimmed or upper case', () => {
    expect(USE_MOCK).toBe(false);
    expect(isRegtestAddress(POCKET)).toBe(true);
    expect(isRegtestAddress(FAUCET)).toBe(true);
    expect(isRegtestAddress(`  ${POCKET}\n`)).toBe(true);
    expect(isRegtestAddress(POCKET.toUpperCase())).toBe(true);
    expect(normalizeAddress(`  ${POCKET.toUpperCase()} `)).toBe(POCKET);
  });

  it('rejects typos, other networks, other kinds, and wrong lengths', () => {
    const typo = `${POCKET.slice(0, 20)}${POCKET[20] === 'q' ? 'p' : 'q'}${POCKET.slice(21)}`;
    expect(isRegtestAddress(typo)).toBe(false);
    expect(isRegtestAddress(`t${POCKET.slice(1)}`)).toBe(false);
    expect(isRegtestAddress(`rzvrg1p${POCKET.slice(7)}`)).toBe(false);
    expect(isRegtestAddress(POCKET.slice(0, -1))).toBe(false);
    expect(isRegtestAddress(`${POCKET}q`)).toBe(false);
    expect(isRegtestAddress(POCKET.replace('y', 'b'))).toBe(false);
    expect(isRegtestAddress('')).toBe(false);
  });
});

describe('faucet requests', () => {
  it('posts the normalized address without cookies or referrer', async () => {
    const calls: Call[] = [];
    const payout = {
      txid: 'ab'.repeat(32),
      address: POCKET,
      amount: '100000000000000000000',
      amount_zvrg: '100',
      fee: '189',
    };
    const result = await requestFaucet(` ${POCKET.toUpperCase()} `, fakeFetch(200, payout, calls));
    expect(result).toEqual({ ok: true, data: payout });
    expect(calls).toHaveLength(1);
    expect(calls[0].url).toBe(FAUCET_URL);
    expect(FAUCET_URL).toBe('https://zevrigo-faucet.ratuaurion.store/api/v1/faucet');
    expect(calls[0].init?.method).toBe('POST');
    expect(calls[0].init?.body).toBe(JSON.stringify({ address: POCKET }));
    expect(calls[0].init?.credentials).toBe('omit');
    expect(calls[0].init?.referrerPolicy).toBe('no-referrer');
  });

  it('maps faucet error codes, bad JSON, and network failures', async () => {
    const limited = await requestFaucet(
      POCKET,
      fakeFetch(429, { error: 'visitor_limit', message: 'x' })
    );
    expect(limited).toEqual({
      ok: false,
      failure: { kind: 'http', status: 429, error: 'visitor_limit' },
    });
    if (!limited.ok) {
      expect(faucetErrorKey(limited.failure)).toBe('faucetErrVisitorLimit');
    }
    const broken = await requestFaucet(POCKET, fakeFetch(502, '<html>'));
    expect(broken.ok).toBe(false);
    if (!broken.ok) {
      expect(broken.failure.kind).toBe('format');
      expect(faucetErrorKey(broken.failure)).toBe('faucetErrUnavailable');
    }
    const offline = await requestFaucet(POCKET, (async () => {
      throw new TypeError('Failed to fetch');
    }) as unknown as FetchImpl);
    expect(offline).toEqual({
      ok: false,
      failure: { kind: 'network', message: 'Failed to fetch' },
    });
  });

  it('reads the faucet status with GET', async () => {
    const calls: Call[] = [];
    const status = {
      network: 'regtest',
      address: FAUCET,
      amount: '100000000000000000000',
      amount_zvrg: '100',
      balance: '0',
      balance_zvrg: '0',
      daily_limit: 100,
      remaining_today: 100,
      synced: true,
      ready: false,
      resets_at_utc: '00:00',
    };
    const result = await fetchFaucetStatus(fakeFetch(200, status, calls));
    expect(result).toEqual({ ok: true, data: status });
    expect(calls[0].init?.method).toBe('GET');
    expect(calls[0].init?.body).toBeUndefined();
  });

  it('has a text for every faucet error code in both languages', () => {
    const codes = [
      'bad_address',
      'own_address',
      'visitor_limit',
      'address_limit',
      'daily_limit',
      'empty',
      'busy',
      'rate_limited',
      'node_unreachable',
      'internal',
    ];
    const keys = codes.map((error) => faucetErrorKey({ kind: 'http', status: 400, error }));
    expect(new Set(keys).size).toBe(9);
    for (const key of keys) {
      expect(id[key].length).toBeGreaterThan(0);
      expect(en[key].length).toBeGreaterThan(0);
    }
  });
});
