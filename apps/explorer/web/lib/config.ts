/**
 * Konfigurasi antarmuka web explorer.
 *
 * Mode data tiruan (`USE_MOCK`) hanya aktif bila build atau server pengembangan dijalankan dengan
 * `NEXT_PUBLIC_USE_MOCK=1` (`npm run dev:mock`): `lib/api.ts` lalu memakai data Regtest tetap dari `lib/mock.ts`
 * sehingga tampilan bisa dilihat tanpa node. Build biasa (`npm run build`, `make web-check`) selalu membaca API
 * explorer sungguhan di origin yang sama (`homeostasis-explorer --web-dir apps/explorer/web/out`).
 */
export const USE_MOCK = process.env.NEXT_PUBLIC_USE_MOCK === '1';

export const API_BASE_PATH = '/api/v1';
export const POLL_INTERVAL_MS = 10_000;
