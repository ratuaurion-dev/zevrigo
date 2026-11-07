/**
 * Pemeriksaan alamat bech32m Zevrigo di antarmuka (MOD-15), sama dengan aturan MOD-09 `decode_address`
 * (Spesifikasi Format Bagian 8): satu jenis huruf, HRP milik jaringan, 59 simbol data, checksum bech32m, jenis 0
 * atau 1, bit sisa nol. Wallet di Rust tetap memeriksa ulang sebelum menyusun transaksi.
 */

export type WalletNetwork = 'testnet' | 'regtest';

export const NETWORKS: readonly WalletNetwork[] = ['testnet', 'regtest'];

export const NETWORK_HRP: Record<WalletNetwork, string> = {
  testnet: 'tzvrg',
  regtest: 'rzvrg',
};

const KNOWN_HRP: Record<string, string> = {
  zvrg: 'mainnet',
  tzvrg: 'testnet',
  rzvrg: 'regtest',
};

const CHARSET = 'qpzry9x8gf2tvdw0s3jn54khce6mua7l';
const BECH32M_CONST = 0x2bc830a3;
const GENERATOR = [0x3b6a57b2, 0x26508e6d, 0x1ea119fa, 0x3d4233dd, 0x2a1462b3];
/** Simbol data: 1 jenis + 52 payload (32 byte) + 6 checksum. */
const DATA_LEN = 59;

export type AddressError =
  | 'empty'
  | 'mixed_case'
  | 'format'
  | 'unknown_prefix'
  | 'wrong_network'
  | 'length'
  | 'checksum'
  | 'kind'
  | 'padding';

export type AddressCheck =
  | { ok: true; address: string }
  | { ok: false; error: AddressError; detected?: string };

function polymod(values: number[]): number {
  let chk = 1;
  for (const value of values) {
    const top = chk >>> 25;
    chk = ((chk & 0x1ffffff) << 5) ^ value;
    GENERATOR.forEach((gen, i) => {
      if ((top >>> i) & 1) {
        chk ^= gen;
      }
    });
  }
  return chk >>> 0;
}

function hrpExpand(hrp: string): number[] {
  const codes = [...hrp].map((c) => c.charCodeAt(0));
  return [...codes.map((c) => c >> 5), 0, ...codes.map((c) => c & 31)];
}

/** Memeriksa alamat penerima untuk `network`. Alamat sah dikembalikan dalam huruf kecil. */
export function checkAddress(input: string, network: WalletNetwork): AddressCheck {
  const text = input.trim();
  if (text === '') {
    return { ok: false, error: 'empty' };
  }
  if (/[a-z]/.test(text) && /[A-Z]/.test(text)) {
    return { ok: false, error: 'mixed_case' };
  }
  const lower = text.toLowerCase();
  const separator = lower.lastIndexOf('1');
  if (separator < 1) {
    return { ok: false, error: 'format' };
  }
  const hrp = lower.slice(0, separator);
  const data = lower.slice(separator + 1);
  const detected = KNOWN_HRP[hrp];
  if (detected === undefined) {
    return { ok: false, error: 'unknown_prefix' };
  }
  if (hrp !== NETWORK_HRP[network]) {
    return { ok: false, error: 'wrong_network', detected };
  }
  if (data.length !== DATA_LEN) {
    return { ok: false, error: 'length' };
  }
  const values = [...data].map((c) => CHARSET.indexOf(c));
  if (values.some((v) => v < 0)) {
    return { ok: false, error: 'format' };
  }
  if (polymod([...hrpExpand(hrp), ...values]) !== BECH32M_CONST) {
    return { ok: false, error: 'checksum' };
  }
  const kind = values[0];
  if (kind !== 0 && kind !== 1) {
    return { ok: false, error: 'kind' };
  }
  const lastPayload = values[DATA_LEN - 7] ?? 0;
  if ((lastPayload & 0x0f) !== 0) {
    return { ok: false, error: 'padding' };
  }
  return { ok: true, address: lower };
}
