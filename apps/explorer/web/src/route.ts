// Jalur hash antarmuka web (Spesifikasi Explorer Web v0.1 Bagian 2) dan pemeriksaan masukan sebelum dikirim ke API.
import type { SearchResult } from './api';

export type Route =
  | { name: 'home' }
  | { name: 'blocks'; before: string | null }
  | { name: 'block'; key: string }
  | { name: 'tx'; txid: string }
  | { name: 'address'; address: string; page: string }
  | { name: 'supply' }
  | { name: 'allocation'; page: string }
  | { name: 'mempool' }
  | { name: 'notfound' };

const DECIMAL = /^(0|[1-9][0-9]{0,19})$/;
const PAGE = /^(0|[1-9][0-9]{0,8})$/;
const HEX64 = /^[0-9a-f]{64}$/;
const ADDRESS = /^(hsis|thsis|rhsis)1[qpzry9x8gf2tvdw0s3jn54khce6mua7l]{6,100}$/;

/** Bilangan desimal kanonis (tinggi blok). */
export function isDecimal(text: string): boolean {
  return DECIMAL.test(text);
}

/** Hash hex huruf kecil 64 karakter (block_id, txid). */
export function isHex64(text: string): boolean {
  return HEX64.test(text);
}

/** Alamat bech32m HSIS (DEC-11): awalan jaringan dan karakter bech32 saja. */
export function isAddress(text: string): boolean {
  return ADDRESS.test(text);
}

/** Masukan pencarian yang boleh dikirim ke `/search/`. */
export function isSearchable(text: string): boolean {
  return isDecimal(text) || isHex64(text) || isAddress(text);
}

function queryParam(query: string, key: string): string | null {
  for (const pair of query.split('&')) {
    const at = pair.indexOf('=');
    const name = at < 0 ? pair : pair.slice(0, at);
    if (name === key) {
      return at < 0 ? '' : pair.slice(at + 1);
    }
  }
  return null;
}

const NOT_FOUND: Route = { name: 'notfound' };

/** Mengurai `location.hash` (`#/…`). */
export function parseRoute(hash: string): Route {
  const text = hash.startsWith('#') ? hash.slice(1) : hash;
  const at = text.indexOf('?');
  const path = at < 0 ? text : text.slice(0, at);
  const query = at < 0 ? '' : text.slice(at + 1);
  if (path === '' || path === '/') {
    return { name: 'home' };
  }
  if (!path.startsWith('/')) {
    return NOT_FOUND;
  }
  const parts = path.slice(1).split('/');
  const page = queryParam(query, 'page') ?? '0';
  if (!PAGE.test(page)) {
    return NOT_FOUND;
  }
  if (parts.length === 1) {
    switch (parts[0]) {
      case 'blocks': {
        const before = queryParam(query, 'before');
        return before === null || isDecimal(before) ? { name: 'blocks', before } : NOT_FOUND;
      }
      case 'supply':
        return { name: 'supply' };
      case 'allocation':
        return { name: 'allocation', page };
      case 'mempool':
        return { name: 'mempool' };
      default:
        return NOT_FOUND;
    }
  }
  if (parts.length === 2) {
    const value = parts[1] ?? '';
    switch (parts[0]) {
      case 'block':
        return isDecimal(value) || isHex64(value) ? { name: 'block', key: value } : NOT_FOUND;
      case 'tx':
        return isHex64(value) ? { name: 'tx', txid: value } : NOT_FOUND;
      case 'address':
        return isAddress(value) ? { name: 'address', address: value, page } : NOT_FOUND;
      default:
        return NOT_FOUND;
    }
  }
  return NOT_FOUND;
}

function withPage(path: string, page: string): string {
  return page === '0' ? path : `${path}?page=${page}`;
}

/** Jalur hash untuk `route`. */
export function href(route: Route): string {
  switch (route.name) {
    case 'home':
      return '#/';
    case 'blocks':
      return route.before === null ? '#/blocks' : `#/blocks?before=${route.before}`;
    case 'block':
      return `#/block/${route.key}`;
    case 'tx':
      return `#/tx/${route.txid}`;
    case 'address':
      return withPage(`#/address/${route.address}`, route.page);
    case 'supply':
      return '#/supply';
    case 'allocation':
      return withPage('#/allocation', route.page);
    case 'mempool':
      return '#/mempool';
    case 'notfound':
      return '#/';
  }
}

/** Jalur hash untuk hasil `/search/`. */
export function searchHref(result: SearchResult): string {
  if (result.kind === 'block' && (isDecimal(result.key) || isHex64(result.key))) {
    return href({ name: 'block', key: result.key });
  }
  if (result.kind === 'tx' && isHex64(result.key)) {
    return href({ name: 'tx', txid: result.key });
  }
  if (result.kind === 'address' && isAddress(result.key)) {
    return href({ name: 'address', address: result.key, page: '0' });
  }
  return href({ name: 'home' });
}
