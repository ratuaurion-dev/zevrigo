import { describe, expect, it } from 'vitest';
import type { Route } from './route';
import { href, isSearchable, parseRoute, searchHref } from './route';

const HASH = 'ab'.repeat(32);
const ADDRESS = 'rhsis1qqqsyqcyq5rqwzqfpg9scrgwpugpzysnzs23v9ccrydpk8qarc0jqszjs4m';

describe('route', () => {
  it('parses every page route', () => {
    expect(parseRoute('')).toEqual({ name: 'home' });
    expect(parseRoute('#/')).toEqual({ name: 'home' });
    expect(parseRoute('#/blocks')).toEqual({ name: 'blocks', before: null });
    expect(parseRoute('#/blocks?before=86')).toEqual({ name: 'blocks', before: '86' });
    expect(parseRoute('#/block/0')).toEqual({ name: 'block', key: '0' });
    expect(parseRoute(`#/block/${HASH}`)).toEqual({ name: 'block', key: HASH });
    expect(parseRoute(`#/tx/${HASH}`)).toEqual({ name: 'tx', txid: HASH });
    expect(parseRoute(`#/address/${ADDRESS}`)).toEqual({ name: 'address', address: ADDRESS, page: '0' });
    expect(parseRoute(`#/address/${ADDRESS}?page=3`)).toEqual({ name: 'address', address: ADDRESS, page: '3' });
    expect(parseRoute('#/supply')).toEqual({ name: 'supply' });
    expect(parseRoute('#/allocation?page=1')).toEqual({ name: 'allocation', page: '1' });
    expect(parseRoute('#/mempool')).toEqual({ name: 'mempool' });
  });

  it('rejects invalid routes', () => {
    const bad = [
      '#/nope',
      '#/block/01',
      '#/block/-1',
      `#/block/${HASH.toUpperCase()}`,
      `#/tx/${HASH.slice(1)}`,
      '#/tx/0',
      '#/address/bc1qxyz',
      `#/address/${ADDRESS}?page=01`,
      '#/blocks?before=x',
      '#/allocation?page=-1',
      `#/block/${HASH}/extra`,
      '#block/1',
    ];
    for (const hash of bad) {
      expect(parseRoute(hash)).toEqual({ name: 'notfound' });
    }
  });

  it('builds hrefs that parse back', () => {
    const routes: Route[] = [
      { name: 'home' },
      { name: 'blocks', before: null },
      { name: 'blocks', before: '20' },
      { name: 'block', key: '7' },
      { name: 'block', key: HASH },
      { name: 'tx', txid: HASH },
      { name: 'address', address: ADDRESS, page: '0' },
      { name: 'address', address: ADDRESS, page: '2' },
      { name: 'supply' },
      { name: 'allocation', page: '0' },
      { name: 'allocation', page: '4' },
      { name: 'mempool' },
    ];
    for (const route of routes) {
      expect(parseRoute(href(route))).toEqual(route);
    }
    expect(href({ name: 'address', address: ADDRESS, page: '0' })).toBe(`#/address/${ADDRESS}`);
  });

  it('validates search input', () => {
    expect(isSearchable('105')).toBe(true);
    expect(isSearchable(HASH)).toBe(true);
    expect(isSearchable(ADDRESS)).toBe(true);
    for (const bad of ['', '1.5', 'abc', `${HASH}0`, 'rhsis1b', '<script>']) {
      expect(isSearchable(bad)).toBe(false);
    }
  });

  it('maps search results to routes', () => {
    expect(searchHref({ kind: 'block', key: HASH })).toBe(`#/block/${HASH}`);
    expect(searchHref({ kind: 'tx', key: HASH })).toBe(`#/tx/${HASH}`);
    expect(searchHref({ kind: 'address', key: ADDRESS })).toBe(`#/address/${ADDRESS}`);
    expect(searchHref({ kind: 'tx', key: 'javascript:alert(1)' })).toBe('#/');
    expect(searchHref({ kind: 'other', key: HASH })).toBe('#/');
  });
});
