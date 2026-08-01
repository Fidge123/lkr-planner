import { useCallback, useEffect, useRef, useState } from "react";
import type { CacheLoadResult } from "../../services/ttl-cache";

export interface CachedResourceState<T> {
  data: T[];
  isLoading: boolean;
  errorMessage: string | null;
  reload: () => void;
}

interface Options<T> {
  load: (options: { forceRefresh: boolean }) => Promise<CacheLoadResult<T>>;
  /** German message for a rejection that carries none of its own. */
  errorMessage: string;
  /** Painted before the load resolves, so a cold start shows the last known data. */
  preload?: () => Promise<T[]>;
}

export function useCachedResource<T>(
  options: Options<T>,
): CachedResourceState<T> {
  const [data, setData] = useState<T[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  // A superseded request must not overwrite a newer one's result, and the
  // options object is rebuilt every render, so both are read through refs.
  const requestIdRef = useRef(0);
  const optionsRef = useRef(options);
  optionsRef.current = options;

  const loadResource = useCallback(async (forceRefresh: boolean) => {
    const { load, preload, errorMessage: fallbackMessage } = optionsRef.current;
    const id = ++requestIdRef.current;
    setIsLoading(true);

    if (!forceRefresh && preload) {
      const preloaded = await preload();
      if (id !== requestIdRef.current) return;
      if (preloaded.length > 0) {
        setData(preloaded);
      }
    }

    try {
      const result = await load({ forceRefresh });
      if (id !== requestIdRef.current) return;
      setData(result.data);
      setErrorMessage(result.errorMessage ?? null);
    } catch (error) {
      if (id !== requestIdRef.current) return;
      setErrorMessage(error instanceof Error ? error.message : fallbackMessage);
    } finally {
      if (id === requestIdRef.current) {
        setIsLoading(false);
      }
    }
  }, []);

  useEffect(() => {
    void loadResource(false);
  }, [loadResource]);

  const reload = useCallback(() => {
    void loadResource(true);
  }, [loadResource]);

  return { data, isLoading, errorMessage, reload };
}
