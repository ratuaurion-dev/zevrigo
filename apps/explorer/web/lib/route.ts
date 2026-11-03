const HEIGHT_REGEX = /^(0|[1-9][0-9]{0,19})$/;
const HASH_REGEX = /^[0-9a-f]{64}$/;
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

export function isValidPage(value: string): boolean {
  return typeof value === 'string' && PAGE_REGEX.test(value);
}

export type ClassifiedQuery =
  | { kind: 'height'; normalized: string }
  | { kind: 'hash'; normalized: string }
  | { kind: 'invalid'; normalized: string };

export function classifySearchQuery(raw: string): ClassifiedQuery {
  const normalized = (raw ?? '').trim().toLowerCase();
  if (isValidHeight(normalized)) {
    return { kind: 'height', normalized };
  }
  if (isValidHash(normalized)) {
    return { kind: 'hash', normalized };
  }
  // Alamat tidak pernah dicari (DEC-28): explorer tidak menampilkan "apa dan siapa".
  return { kind: 'invalid', normalized };
}

export const routes = {
  home: () => '/',
  blocks: (before?: number) =>
    before !== undefined ? `/blocks/?before=${encodeURIComponent(String(before))}` : '/blocks/',
  block: (idOrHeight: string | number) =>
    `/block/?id=${encodeURIComponent(String(idOrHeight))}`,
  tx: (txid: string) => `/tx/?id=${encodeURIComponent(txid)}`,
  supply: () => '/supply/',
  mempool: () => '/mempool/',
  search: (query: string) => `/search/?q=${encodeURIComponent(query.trim())}`,
};

export function truncateMiddle(str: string, startChars = 10, endChars = 8): string {
  if (!str || str.length <= startChars + endChars + 3) {
    return str;
  }
  return `${str.slice(0, startChars)}…${str.slice(-endChars)}`;
}
