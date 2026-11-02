// Memuat data API untuk satu halaman, dengan pembaruan berkala opsional.
import { useEffect, useState } from 'react';
import type { Failure, Result } from './api';

export type Load<T> = { status: 'loading' } | { status: 'ok'; data: T } | { status: 'error'; failure: Failure };

/** Selang pembaruan beranda dan status (Spesifikasi Explorer Web v0.1 Bagian 2). */
export const REFRESH_MS = 10_000;

/**
 * Menjalankan `load` setiap `key` berubah atau `reload` dipanggil, dan setiap `refreshMs` jika bukan null.
 * Pembaruan berkala tidak menampilkan "memuat" lagi; data lama tetap tampil sampai data baru datang.
 */
export function useLoad<T>(
  load: () => Promise<Result<T>>,
  key: string,
  refreshMs: number | null,
): { load: Load<T>; reload: () => void } {
  const [state, setState] = useState<Load<T>>({ status: 'loading' });
  const [round, setRound] = useState(0);
  useEffect(() => {
    let active = true;
    const run = () => {
      void load().then((result) => {
        if (active) {
          setState(result.ok ? { status: 'ok', data: result.data } : { status: 'error', failure: result.failure });
        }
      });
    };
    setState({ status: 'loading' });
    run();
    const timer = refreshMs === null ? null : window.setInterval(run, refreshMs);
    return () => {
      active = false;
      if (timer !== null) {
        window.clearInterval(timer);
      }
    };
  }, [key, round]);
  return { load: state, reload: () => setRound((value) => value + 1) };
}
