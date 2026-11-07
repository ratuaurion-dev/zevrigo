import { describe, expect, test } from 'vitest';
import { checkAddress } from './address';

// W5, W10: Spesifikasi Wallet v0.1 Bagian 4. TESTNET_0, MAINNET_0: Spesifikasi Format Bagian 8. Alamat regtest
// jenis 1, jenis 2, satu simbol kurang, dan bit sisa bukan nol: payload W5 dengan checksum bech32m yang benar.
const W5 = 'rzvrg1qtfhjsfflxh307x4skzrt3jrlwvg4jes32ljmut9fsxyrd939mseq59qlxg';
const W10 = 'tzvrg1qtfhjsfflxh307x4skzrt3jrlwvg4jes32ljmut9fsxyrd939mseq4f4dkt';
const TESTNET_0 = 'tzvrg1qlycg5qvjtrp3qjf5f7zl382j9x6nrjz9sdhenvyxq8c3808qxmusgxrxjr';
const MAINNET_0 = 'zvrg1qlycg5qvjtrp3qjf5f7zl382j9x6nrjz9sdhenvyxq8c3808qxmush8h94g';
const REG_KIND1 = 'rzvrg1ptfhjsfflxh307x4skzrt3jrlwvg4jes32ljmut9fsxyrd939mseqtws6mk';
const REG_KIND2 = 'rzvrg1ztfhjsfflxh307x4skzrt3jrlwvg4jes32ljmut9fsxyrd939mseqrnf44a';
const REG_SHORT = 'rzvrg1qtfhjsfflxh307x4skzrt3jrlwvg4jes32ljmut9fsxyrd939mse2h98j2';
const REG_BAD_PADDING = 'rzvrg1qtfhjsfflxh307x4skzrt3jrlwvg4jes32ljmut9fsxyrd939msepfn52m6';

describe('alamat', () => {
  test('menerima alamat sah jaringan sendiri', () => {
    expect(checkAddress(W5, 'regtest')).toEqual({ ok: true, address: W5 });
    expect(checkAddress(` ${W10} `, 'testnet')).toEqual({ ok: true, address: W10 });
    expect(checkAddress(TESTNET_0, 'testnet')).toEqual({ ok: true, address: TESTNET_0 });
    expect(checkAddress(W10.toUpperCase(), 'testnet')).toEqual({ ok: true, address: W10 });
    expect(checkAddress(REG_KIND1, 'regtest')).toEqual({ ok: true, address: REG_KIND1 });
  });

  test('menolak alamat jaringan lain dan awalan asing', () => {
    expect(checkAddress(W5, 'testnet')).toEqual({ ok: false, error: 'wrong_network', detected: 'regtest' });
    expect(checkAddress(MAINNET_0, 'testnet')).toEqual({ ok: false, error: 'wrong_network', detected: 'mainnet' });
    expect(checkAddress('bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq', 'testnet')).toEqual({
      ok: false,
      error: 'unknown_prefix',
    });
    expect(checkAddress('', 'testnet')).toEqual({ ok: false, error: 'empty' });
    expect(checkAddress('tzvrg1QLYCG5', 'testnet')).toEqual({ ok: false, error: 'mixed_case' });
    expect(checkAddress('1qlycg5', 'testnet')).toEqual({ ok: false, error: 'format' });
  });

  test('menolak panjang, simbol, checksum, jenis, dan bit sisa yang salah', () => {
    expect(checkAddress(REG_SHORT, 'regtest')).toEqual({ ok: false, error: 'length' });
    expect(checkAddress(W5.slice(0, -1) + 'h', 'regtest')).toEqual({ ok: false, error: 'checksum' });
    expect(checkAddress(W5.slice(0, -1) + 'b', 'regtest')).toEqual({ ok: false, error: 'format' });
    expect(checkAddress(REG_KIND2, 'regtest')).toEqual({ ok: false, error: 'kind' });
    expect(checkAddress(REG_BAD_PADDING, 'regtest')).toEqual({ ok: false, error: 'padding' });
  });
});
