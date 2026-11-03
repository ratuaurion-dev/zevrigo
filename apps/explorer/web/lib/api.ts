import { API_BASE_PATH, USE_MOCK } from './config';
import {
  getActiveMockOverride,
  getMockBlockDetail,
  getMockBlocks,
  getMockMempoolInfo,
  getMockSearchResult,
  getMockStatus,
  getMockSupplyInfo,
  getMockTxDetail,
} from './mock';

export type Status = {
  network: string;
  indexed_height: number | null;
  indexed_block_id: string | null;
  node_height: number | null;
  node_best_block_id: string | null;
  chainwork: string | null;
  mempool_count: number;
  index_error: string | null;
  last_error: string | null;
};

export type BlockRow = {
  height: number;
  block_id: string;
  timestamp: number;
  size: number;
  tx_count: number;
  confirmations: number;
};

export type BlockList = {
  blocks: BlockRow[];
};

export type BlockDetail = {
  block_id: string;
  height: number;
  prev_block_id: string;
  merkle_root: string;
  timestamp: number;
  target: string;
  nonce: string;
  size: number;
  tx_count: number;
  subsidy: string;
  fees: string;
  confirmations: number;
  txids: string[];
  next_block_id: string | null;
};

export type TxDetail = {
  txid: string;
  block_id: string;
  height: number;
  position: number;
  size: number;
  is_coinbase: boolean;
  fee: string;
  confirmations: number;
  /** Jumlah input dan output saja: API tidak menyajikan alamat, jumlah per output, maupun jejak koin (DEC-28). */
  input_count: number;
  output_count: number;
};

export type SupplyInfo = {
  height: number;
  mined_total: string;
  scheduled_mined_total: string;
  matches_schedule: boolean;
  fees_total: string;
  next_subsidy: string;
  tip_target: string;
  average_block_seconds: number | null;
};

export type MempoolInfo = {
  count: number;
};

export type SearchResult = {
  kind: 'block' | 'tx';
  key: string;
};

export type ApiFailure =
  | {
      kind: 'http';
      status: number;
      error: string;
      inMempool?: boolean;
    }
  | {
      kind: 'network';
      message: string;
    }
  | {
      kind: 'format';
      message: string;
    };

export type ApiResult<T> =
  | { ok: true; data: T }
  | { ok: false; failure: ApiFailure };

export type FetchImpl = typeof fetch;

/**
 * Low-level safe HTTP GET helper that never throws.
 * Maps HTTP errors (including `in_mempool` on 404), network failures, and malformed JSON
 * into typed `ApiResult<T>` values.
 */
export async function requestJson<T>(
  path: string,
  fetchFn: FetchImpl = fetch
): Promise<ApiResult<T>> {
  try {
    const response = await fetchFn(`${API_BASE_PATH}${path}`, {
      method: 'GET',
      headers: {
        Accept: 'application/json',
      },
    });

    let parsed: unknown;
    try {
      parsed = await response.json();
    } catch {
      return {
        ok: false,
        failure: {
          kind: 'format',
          message: `Invalid JSON response (HTTP ${response.status})`,
        },
      };
    }

    if (!response.ok) {
      const bodyObj =
        typeof parsed === 'object' && parsed !== null
          ? (parsed as Record<string, unknown>)
          : {};
      const errCode = typeof bodyObj.error === 'string' ? bodyObj.error : `http_${response.status}`;
      const inMempool = typeof bodyObj.in_mempool === 'boolean' ? bodyObj.in_mempool : undefined;
      return {
        ok: false,
        failure: {
          kind: 'http',
          status: response.status,
          error: errCode,
          ...(inMempool !== undefined ? { inMempool } : {}),
        },
      };
    }

    if (typeof parsed !== 'object' || parsed === null) {
      return {
        ok: false,
        failure: {
          kind: 'format',
          message: 'Expected JSON object payload',
        },
      };
    }

    return {
      ok: true,
      data: parsed as T,
    };
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

async function mockDelay(): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, 60));
}

function checkMockGlobalFailure(endpointIsStatus = false): ApiResult<never> | null {
  const override = getActiveMockOverride();
  if (override === 'offline') {
    return {
      ok: false,
      failure: {
        kind: 'network',
        message: 'Simulated offline connection failure (?mock=offline)',
      },
    };
  }
  if (override === 'notready' && !endpointIsStatus) {
    return {
      ok: false,
      failure: {
        kind: 'http',
        status: 503,
        error: 'not_ready',
      },
    };
  }
  return null;
}

export async function fetchStatus(fetchFn?: FetchImpl): Promise<ApiResult<Status>> {
  if (USE_MOCK && !fetchFn) {
    await mockDelay();
    const fail = checkMockGlobalFailure(true);
    if (fail) return fail;
    return { ok: true, data: getMockStatus() };
  }
  return requestJson<Status>('/status', fetchFn);
}

export async function fetchBlocks(
  before?: number,
  fetchFn?: FetchImpl
): Promise<ApiResult<BlockList>> {
  if (USE_MOCK && !fetchFn) {
    await mockDelay();
    const fail = checkMockGlobalFailure(false);
    if (fail) return fail;
    return { ok: true, data: getMockBlocks(before) };
  }
  const query =
    before !== undefined ? `?before=${encodeURIComponent(String(before))}` : '';
  return requestJson<BlockList>(`/blocks${query}`, fetchFn);
}

export async function fetchBlock(
  idOrHeight: string | number,
  fetchFn?: FetchImpl
): Promise<ApiResult<BlockDetail>> {
  const key = String(idOrHeight);
  if (USE_MOCK && !fetchFn) {
    await mockDelay();
    const fail = checkMockGlobalFailure(false);
    if (fail) return fail;
    const detail = getMockBlockDetail(key);
    if (!detail) {
      return {
        ok: false,
        failure: { kind: 'http', status: 404, error: 'not_found' },
      };
    }
    return { ok: true, data: detail };
  }
  return requestJson<BlockDetail>(`/block/${encodeURIComponent(key)}`, fetchFn);
}

export async function fetchTx(
  txid: string,
  fetchFn?: FetchImpl
): Promise<ApiResult<TxDetail>> {
  if (USE_MOCK && !fetchFn) {
    await mockDelay();
    const fail = checkMockGlobalFailure(false);
    if (fail) return fail;
    const { tx, inMempool } = getMockTxDetail(txid);
    if (!tx) {
      return {
        ok: false,
        failure: {
          kind: 'http',
          status: 404,
          error: 'not_found',
          inMempool,
        },
      };
    }
    return { ok: true, data: tx };
  }
  return requestJson<TxDetail>(`/tx/${encodeURIComponent(txid)}`, fetchFn);
}

export async function fetchSupply(fetchFn?: FetchImpl): Promise<ApiResult<SupplyInfo>> {
  if (USE_MOCK && !fetchFn) {
    await mockDelay();
    const fail = checkMockGlobalFailure(false);
    if (fail) return fail;
    return { ok: true, data: getMockSupplyInfo() };
  }
  return requestJson<SupplyInfo>('/supply', fetchFn);
}

export async function fetchMempool(fetchFn?: FetchImpl): Promise<ApiResult<MempoolInfo>> {
  if (USE_MOCK && !fetchFn) {
    await mockDelay();
    const fail = checkMockGlobalFailure(false);
    if (fail) return fail;
    return { ok: true, data: getMockMempoolInfo() };
  }
  return requestJson<MempoolInfo>('/mempool', fetchFn);
}

export async function fetchSearch(
  query: string,
  fetchFn?: FetchImpl
): Promise<ApiResult<SearchResult>> {
  if (USE_MOCK && !fetchFn) {
    await mockDelay();
    const fail = checkMockGlobalFailure(false);
    if (fail) return fail;
    const { result, inMempool } = getMockSearchResult(query);
    if (!result) {
      return {
        ok: false,
        failure: {
          kind: 'http',
          status: 404,
          error: 'not_found',
          ...(inMempool ? { inMempool: true } : { inMempool: false }),
        },
      };
    }
    return { ok: true, data: result };
  }
  return requestJson<SearchResult>(`/search/${encodeURIComponent(query)}`, fetchFn);
}
