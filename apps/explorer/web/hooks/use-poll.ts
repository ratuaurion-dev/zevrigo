'use client';

import { useCallback, useEffect, useRef, useState } from 'react';
import type { ApiFailure, ApiResult } from '@/lib/api';
import { POLL_INTERVAL_MS } from '@/lib/config';

export interface PollState<T> {
  data: T | null;
  failure: ApiFailure | null;
  isLoading: boolean;
  isRefreshing: boolean;
  nowUnix: number;
  reload: () => void;
}

/**
 * Polls an asynchronous `ApiResult<T>` fetcher every `intervalMs` (default 10s).
 * Keeps existing `data` rendered during background refreshes so the UI never flickers.
 */
export function usePoll<T>(
  fetcher: () => Promise<ApiResult<T>>,
  deps: readonly unknown[] = [],
  intervalMs: number = POLL_INTERVAL_MS
): PollState<T> {
  const [data, setData] = useState<T | null>(null);
  const [failure, setFailure] = useState<ApiFailure | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [isRefreshing, setIsRefreshing] = useState<boolean>(false);
  const [nowUnix, setNowUnix] = useState<number>(() => Math.trunc(Date.now() / 1000));

  const mountedRef = useRef(true);
  const fetcherRef = useRef(fetcher);
  fetcherRef.current = fetcher;

  const hasDataRef = useRef(false);

  const execute = useCallback(async (isInitial: boolean) => {
    if (!mountedRef.current) return;
    if (isInitial && !hasDataRef.current) {
      setIsLoading(true);
    } else {
      setIsRefreshing(true);
    }

    const res = await fetcherRef.current();
    if (!mountedRef.current) return;

    setNowUnix(Math.trunc(Date.now() / 1000));

    if (res.ok) {
      hasDataRef.current = true;
      setData(res.data);
      setFailure(null);
    } else {
      setFailure(res.failure);
    }
    setIsLoading(false);
    setIsRefreshing(false);
  }, []);

  useEffect(() => {
    mountedRef.current = true;
    hasDataRef.current = false;
    setData(null);
    setFailure(null);
    setIsLoading(true);

    void execute(true);

    if (intervalMs <= 0) {
      return () => {
        mountedRef.current = false;
      };
    }

    const timer = setInterval(() => {
      void execute(false);
    }, intervalMs);

    return () => {
      mountedRef.current = false;
      clearInterval(timer);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [...deps, intervalMs, execute]);

  const reload = useCallback(() => {
    void execute(false);
  }, [execute]);

  return {
    data,
    failure,
    isLoading,
    isRefreshing,
    nowUnix,
    reload,
  };
}
