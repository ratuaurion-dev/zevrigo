const HEIGHT_REGEX = /^(0|[1-9][0-9]{0,19})$/;
const HASH_REGEX = /^[0-9a-f]{64}$/;
const ADDRESS_REGEX = /^(hsis|thsis|rhsis)1[qpzry9x8gf2tvdw0s3jn54khce6mua7l]{6,100}$/;
const PAGE_REGEX = /^(0|[1-9][0-9]{0,8})$/;

export function isValidHeight(value: string): boolean {
  return typeof value === 'string' && HEIGHT_REGEX.test(value);
}

export function isValidHash(value: string): boolean {
  return typeof value === 'string' && HASH_REGEX.test(value);
}

export function isValidBlockIdOrHeight(value: string): boolean {
  return isValidHeight(value) || isValidHash(value);
}

export function isValidAddress(value: string): boolean {
  return typeof value === 'string' && ADDRESS_REGEX.test(value);
}

export function isValidPage(value: string): boolean {
  return typeof value === 'string' && PAGE_REGEX.test(value);
}

export type ClassifiedQuery =
  | { kind: 'height'; normalized: string }
  | { kind: 'hash'; normalized: string }
  | { kind: 'address'; normalized: string }
  | { kind: 'invalid'; normalized: string };

export function classifySearchQuery(raw: string): ClassifiedQuery {
  const normalized = (raw ?? '').trim().toLowerCase();
  if (isValidHeight(normalized)) {
    return { kind: 'height', normalized };
  }
  if (isValidHash(normalized)) {
    return { kind: 'hash', normalized };
  }
  if (isValidAddress(normalized)) {
    return { kind: 'address', normalized };
  }
  return { kind: 'invalid', normalized };
}

export const routes = {
  home: () => '/',
  blocks: (before?: number) =>
    before !== undefined ? `/blocks/?before=${encodeURIComponent(String(before))}` : '/blocks/',
  block: (idOrHeight: string | number) =>
    `/block/?id=${encodeURIComponent(String(idOrHeight))}`,
  tx: (txid: string) => `/tx/?id=${encodeURIComponent(txid)}`,
  address: (address: string, page?: number) => {
    const base = `/address/?a=${encodeURIComponent(address)}`;
    return page !== undefined && page > 0 ? `${base}&page=${encodeURIComponent(String(page))}` : base;
  },
  supply: () => '/supply/',
  allocation: (page?: number) =>
    page !== undefined && page > 0
      ? `/allocation/?page=${encodeURIComponent(String(page))}`
      : '/allocation/',
  mempool: () => '/mempool/',
  nodes: () => '/nodes/',
  search: (query: string) => `/search/?q=${encodeURIComponent(query.trim())}`,
};

export function truncateMiddle(str: string, startChars = 10, endChars = 8): string {
  if (!str || str.length <= startChars + endChars + 3) {
    return str;
  }
  return `${str.slice(0, startChars)}…${str.slice(-endChars)}`;
}
