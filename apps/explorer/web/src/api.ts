// Klien API explorer v0.1 (Spesifikasi Explorer v0.1 Bagian 5). Jumlah koin tetap string desimal quanta.

export interface SpentBy {
  txid: string;
  input: number;
  height: number;
}

export interface Status {
  network: string;
  indexed_height: number | null;
  indexed_block_id: string | null;
  node_height: number | null;
  node_best_block_id: string | null;
  chainwork: string | null;
  mempool_count: number;
  index_error: string | null;
  last_error: string | null;
}

export interface BlockRow {
  height: number;
  block_id: string;
  timestamp: number;
  size: number;
  tx_count: number;
  confirmations: number;
}

export interface BlockList {
  blocks: BlockRow[];
}

export interface BlockDetail {
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
}

export interface TxInput {
  txid: string;
  index: number;
  amount: string;
  kind: number;
  payload: string;
  address: string;
}

export interface TxOutput {
  index: number;
  amount: string;
  kind: number;
  payload: string;
  address: string;
  spent_by: SpentBy | null;
}

export interface TxDetail {
  txid: string;
  block_id: string;
  height: number;
  position: number;
  size: number;
  is_coinbase: boolean;
  fee: string;
  confirmations: number;
  inputs: TxInput[];
  outputs: TxOutput[];
}

export interface AddressTx {
  txid: string;
  height: number;
  confirmations: number;
  received: string;
  sent: string;
}

export interface AddressInfo {
  address: string;
  kind: number;
  payload: string;
  balance: string;
  received: string;
  sent: string;
  utxo_count: number;
  tx_count: number;
  page: number;
  page_size: number;
  txs: AddressTx[];
}

export interface SupplyInfo {
  height: number;
  mined_total: string;
  scheduled_mined_total: string;
  matches_schedule: boolean;
  genesis_allocation: string;
  total_supply: string;
  fees_total: string;
  next_subsidy: string;
  tip_target: string;
  average_block_seconds: number | null;
}

export interface AllocationOutput extends TxOutput {
  txid: string;
}

export interface AllocationInfo {
  genesis_block_id: string;
  allocated: string;
  unspent: string;
  outputs: AllocationOutput[];
  addresses: AddressInfo[];
}

export interface MempoolInfo {
  count: number;
  txids: string[];
}

export interface SearchResult {
  kind: string;
  key: string;
}

/** Kegagalan permintaan: jawaban galat HTTP dari API, jaringan putus, atau badan bukan JSON. */
export type Failure =
  | { kind: 'http'; status: number; error: string; inMempool: boolean }
  | { kind: 'network' }
  | { kind: 'format' };

export type Result<T> = { ok: true; data: T } | { ok: false; failure: Failure };

export type FetchFn = (path: string) => Promise<Response>;

/** Awalan semua jalur API. */
export const API_BASE = '/api/v1';

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

/** GET satu jalur API dan membaca badan JSON-nya. Tidak pernah melempar. */
export async function getJson<T>(fetchFn: FetchFn, path: string): Promise<Result<T>> {
  let response: Response;
  try {
    response = await fetchFn(path);
  } catch {
    return { ok: false, failure: { kind: 'network' } };
  }
  let body: unknown;
  try {
    body = await response.json();
  } catch {
    return { ok: false, failure: { kind: 'format' } };
  }
  if (response.ok) {
    return { ok: true, data: body as T };
  }
  const record = isRecord(body) ? body : {};
  return {
    ok: false,
    failure: {
      kind: 'http',
      status: response.status,
      error: typeof record.error === 'string' ? record.error : '',
      inMempool: record.in_mempool === true,
    },
  };
}

export interface Api {
  status(): Promise<Result<Status>>;
  blocks(before: string | null): Promise<Result<BlockList>>;
  block(key: string): Promise<Result<BlockDetail>>;
  tx(txid: string): Promise<Result<TxDetail>>;
  address(address: string, page: string): Promise<Result<AddressInfo>>;
  supply(): Promise<Result<SupplyInfo>>;
  allocation(page: string): Promise<Result<AllocationInfo>>;
  mempool(): Promise<Result<MempoolInfo>>;
  search(query: string): Promise<Result<SearchResult>>;
}

function segment(value: string): string {
  return encodeURIComponent(value);
}

/** Klien API di atas `fetchFn` (di browser: `fetch` ke origin yang sama). */
export function createApi(fetchFn: FetchFn): Api {
  function get<T>(path: string): Promise<Result<T>> {
    return getJson<T>(fetchFn, `${API_BASE}${path}`);
  }
  return {
    status: () => get<Status>('/status'),
    blocks: (before) => get<BlockList>(before === null ? '/blocks' : `/blocks?before=${segment(before)}`),
    block: (key) => get<BlockDetail>(`/block/${segment(key)}`),
    tx: (txid) => get<TxDetail>(`/tx/${segment(txid)}`),
    address: (address, page) => get<AddressInfo>(`/address/${segment(address)}?page=${segment(page)}`),
    supply: () => get<SupplyInfo>('/supply'),
    allocation: (page) => get<AllocationInfo>(`/allocation?page=${segment(page)}`),
    mempool: () => get<MempoolInfo>('/mempool'),
    search: (query) => get<SearchResult>(`/search/${segment(query)}`),
  };
}

/** Klien API browser: origin yang sama, tanpa cookie. */
export function browserApi(): Api {
  return createApi((path) =>
    fetch(path, { headers: { Accept: 'application/json' }, credentials: 'omit', referrerPolicy: 'no-referrer' }),
  );
}
