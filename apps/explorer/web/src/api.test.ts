import { describe, expect, it } from 'vitest';
import { createApi } from './api';

const HASH = 'cd'.repeat(32);

function json(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), { status, headers: { 'Content-Type': 'application/json' } });
}

describe('api', () => {
  it('builds API paths', async () => {
    const paths: string[] = [];
    const api = createApi(async (path) => {
      paths.push(path);
      return json(200, {});
    });
    await api.status();
    await api.blocks(null);
    await api.blocks('86');
    await api.block('5');
    await api.tx(HASH);
    await api.address('rhsis1abc', '2');
    await api.supply();
    await api.allocation('0');
    await api.mempool();
    await api.search('a/b?c');
    expect(paths).toEqual([
      '/api/v1/status',
      '/api/v1/blocks',
      '/api/v1/blocks?before=86',
      '/api/v1/block/5',
      `/api/v1/tx/${HASH}`,
      '/api/v1/address/rhsis1abc?page=2',
      '/api/v1/supply',
      '/api/v1/allocation?page=0',
      '/api/v1/mempool',
      '/api/v1/search/a%2Fb%3Fc',
    ]);
  });

  it('parses data and errors with in_mempool', async () => {
    const api = createApi(async (path) => {
      if (path.endsWith('/mempool')) {
        return json(200, { count: 1, txids: [HASH] });
      }
      if (path.endsWith('/status')) {
        return json(503, { error: 'not_ready' });
      }
      return json(404, { error: 'not_found', in_mempool: true });
    });
    expect(await api.mempool()).toEqual({ ok: true, data: { count: 1, txids: [HASH] } });
    expect(await api.tx(HASH)).toEqual({
      ok: false,
      failure: { kind: 'http', status: 404, error: 'not_found', inMempool: true },
    });
    expect(await api.status()).toEqual({
      ok: false,
      failure: { kind: 'http', status: 503, error: 'not_ready', inMempool: false },
    });
  });

  it('reports network and format failures', async () => {
    const offline = createApi(async () => {
      throw new TypeError('network down');
    });
    expect(await offline.status()).toEqual({ ok: false, failure: { kind: 'network' } });
    const broken = createApi(async () => new Response('not json', { status: 200 }));
    expect(await broken.status()).toEqual({ ok: false, failure: { kind: 'format' } });
  });
});
