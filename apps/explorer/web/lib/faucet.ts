/**
 * Klien faucet jaringan demo regtest (TSK-M54, `zevrigo-demo faucet`).
 *
 * Halaman faucet memanggil layanan faucet di origin lain ([`FAUCET_URL`]); program explorer mengizinkannya lewat
 * `--faucet-origin` (kebijakan keamanan konten `connect-src`), dan layanan faucet hanya memberi kepala CORS untuk origin
 * explorer. Permintaan dikirim tanpa kuki dan tanpa referrer. Alamat diperiksa di sini (bech32m regtest jenis 0) agar
 * salah ketik langsung terlihat; layanan faucet tetap memeriksanya sendiri.
 */
import type { ApiFailure, ApiResult, FetchImpl } from './api';
import { FAUCET_URL, USE_MOCK } from './config';
import type { DictionaryKey } from './i18n';

export type FaucetStatus = {
  network: string;
  address: string;
  amount: string;
  amount_zvrg: string;
  balance: string;
  balance_zvrg: string;
  daily_limit: number;
  remaining_today: number;
  synced: boolean;
  ready: boolean;
  resets_at_utc: string;
};

export type FaucetPayout = {
  txid: string;
  address: string;
  amount: string;
  amount_zvrg: string;
  fee: string;
};

/** Kode galat bila build ini tanpa alamat faucet (`NEXT_PUBLIC_FAUCET_URL` kosong). */
export const FAUCET_UNAVAILABLE = 'faucet_unavailable';

const BECH32_CHARSET = 'qpzry9x8gf2tvdw0s3jn54khce6mua7l';
const BECH32M_CONST = 0x2bc830a3;
const REGTEST_HRP = 'rzvrg';
/** `rzvrg1` + versi `q` (jenis 0) + 52 karakter payload + 6 karakter checksum. */
const REGTEST_SHAPE = new RegExp(`^${REGTEST_HRP}1q[${BECH32_CHARSET}]{58}$`);

/** Alamat dirapikan: spasi di tepi dibuang, huruf kecil. */
export function normalizeAddress(raw: string): string {
  return (raw ?? '').trim().toLowerCase();
}

function polymod(values: number[]): number {
  const generator = [0x3b6a57b2, 0x26508e6d, 0x1ea119fa, 0x3d4233dd, 0x2a1462b3];
  let check = 1;
  for (const value of values) {
    const top = check >>> 25;
    check = ((check & 0x1ffffff) << 5) ^ value;
    for (let i = 0; i < 5; i += 1) {
      if ((top >>> i) & 1) {
        check ^= generator[i];
      }
    }
  }
  return check >>> 0;
}

/** `true` untuk alamat regtest jenis 0 (`rzvrg1q…`) dengan checksum bech32m yang benar. */
export function isRegtestAddress(raw: string): boolean {
  const address = normalizeAddress(raw);
  if (!REGTEST_SHAPE.test(address)) {
    return false;
  }
  const data = [...address.slice(REGTEST_HRP.length + 1)].map((char) =>
    BECH32_CHARSET.indexOf(char)
  );
  const hrp = [...REGTEST_HRP].map((char) => char.charCodeAt(0));
  const expanded = [...hrp.map((code) => code >> 5), 0, ...hrp.map((code) => code & 31)];
  return polymod([...expanded, ...data]) === BECH32M_CONST;
}

/** Teks untuk kode galat faucet (atau kegagalan jaringan). */
export function faucetErrorKey(failure: ApiFailure): DictionaryKey {
  if (failure.kind !== 'http') {
    return 'faucetErrUnavailable';
  }
  switch (failure.error) {
    case 'bad_address':
      return 'faucetErrBadAddress';
    case 'own_address':
      return 'faucetErrOwnAddress';
    case 'visitor_limit':
      return 'faucetErrVisitorLimit';
    case 'address_limit':
      return 'faucetErrAddressLimit';
    case 'daily_limit':
      return 'faucetErrDailyLimit';
    case 'empty':
      return 'faucetErrEmpty';
    case 'busy':
      return 'faucetErrBusy';
    case 'rate_limited':
      return 'faucetErrRateLimited';
    default:
      return 'faucetErrUnavailable';
  }
}

async function faucetRequest<T>(init: RequestInit, fetchFn: FetchImpl): Promise<ApiResult<T>> {
  if (!FAUCET_URL) {
    return { ok: false, failure: { kind: 'http', status: 0, error: FAUCET_UNAVAILABLE } };
  }
  try {
    const response = await fetchFn(FAUCET_URL, {
      ...init,
      credentials: 'omit',
      cache: 'no-store',
      referrerPolicy: 'no-referrer',
    });
    let parsed: unknown;
    try {
      parsed = await response.json();
    } catch {
      return {
        ok: false,
        failure: { kind: 'format', message: `Invalid JSON response (HTTP ${response.status})` },
      };
    }
    const body =
      typeof parsed === 'object' && parsed !== null ? (parsed as Record<string, unknown>) : null;
    if (!response.ok) {
      const error = body && typeof body.error === 'string' ? body.error : `http_${response.status}`;
      return { ok: false, failure: { kind: 'http', status: response.status, error } };
    }
    if (!body) {
      return { ok: false, failure: { kind: 'format', message: 'Expected JSON object payload' } };
    }
    return { ok: true, data: body as T };
  } catch (err) {
    return {
      ok: false,
      failure: {
        kind: 'network',
        message: err instanceof Error ? err.message : 'Network request failed',
      },
    };
  }
}

const MOCK_STATUS: FaucetStatus = {
  network: 'regtest',
  address: 'rzvrg1qgjknjtac9n0jv3gvy7ffr76l6pld4kms3nkwzf8kk3204yfqt2qs4jxn6k',
  amount: '100000000000000000000',
  amount_zvrg: '100',
  balance: '9899999999999999999811',
  balance_zvrg: '9899.999999999999999811',
  daily_limit: 100,
  remaining_today: 99,
  synced: true,
  ready: true,
  resets_at_utc: '00:00',
};

/** Keadaan faucet (`GET`). */
export async function fetchFaucetStatus(fetchFn?: FetchImpl): Promise<ApiResult<FaucetStatus>> {
  if (USE_MOCK && !fetchFn) {
    return { ok: true, data: MOCK_STATUS };
  }
  return faucetRequest<FaucetStatus>(
    { method: 'GET', headers: { Accept: 'application/json' } },
    fetchFn ?? fetch
  );
}

/** Meminta koin demo ke `address` (`POST`). Alamat dikirim dalam bentuk rapi ([`normalizeAddress`]). */
export async function requestFaucet(
  address: string,
  fetchFn?: FetchImpl
): Promise<ApiResult<FaucetPayout>> {
  const normalized = normalizeAddress(address);
  if (USE_MOCK && !fetchFn) {
    return {
      ok: true,
      data: {
        txid: '40fc2741fc633373f55519885c5febf89d55c779041eab8a1888abcd3a4ca35d',
        address: normalized,
        amount: MOCK_STATUS.amount,
        amount_zvrg: MOCK_STATUS.amount_zvrg,
        fee: '189',
      },
    };
  }
  return faucetRequest<FaucetPayout>(
    {
      method: 'POST',
      headers: { Accept: 'application/json', 'Content-Type': 'application/json' },
      body: JSON.stringify({ address: normalized }),
    },
    fetchFn ?? fetch
  );
}
