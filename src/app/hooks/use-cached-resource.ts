import { useCallback, useEffect, useRef, useState } from "react";
import type { CacheLoadResult } from "../../services/ttl-cache";

export interface CachedResourceState<T> {
  data: T[];
  isLoading: boolean;
  errorMessage: string | null;
  reload: () => void;
}

export interface ResourceLoadSpec<T> {
  load: (options: { forceRefresh: boolean }) => Promise<CacheLoadResult<T>>;
  /** German message for a rejection that carries none of its own. */
  errorMessage: string;
  /** Painted before the load resolves, so a cold start shows the last known data. */
  preload?: () => Promise<T[]>;
}

export interface ResourceUpdate<T> {
  data?: T[];
  errorMessage?: string | null;
  isLoading?: boolean;
}

/**
 * A failing preload is an optimisation that missed, never a user-facing error, so it
 * cannot reject: an escaping rejection would skip the `isLoading: false` update and
 * leave the caller loading forever.
 */
export async function runResourceLoad<T>(
  { load, preload, errorMessage }: ResourceLoadSpec<T>,
  forceRefresh: boolean,
  emit: (update: ResourceUpdate<T>) => void,
): Promise<void> {
  if (!forceRefresh && preload) {
    const preloaded = await preload().catch(() => []);
    if (preloaded.length > 0) {
      emit({ data: preloaded });
    }
  }

  try {
    const result = await load({ forceRefresh });
    emit({ data: result.data, errorMessage: result.errorMessage ?? null });
  } catch (error) {
    emit({
      errorMessage: error instanceof Error ? error.message : errorMessage,
    });
  } finally {
    emit({ isLoading: false });
  }
}

export function useCachedResource<T>(
  spec: ResourceLoadSpec<T>,
): CachedResourceState<T> {
  const [data, setData] = useState<T[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  // A superseded request must not overwrite a newer one's result, and the
  // spec object is rebuilt every render, so both are read through refs.
  const requestIdRef = useRef(0);
  const specRef = useRef(spec);
  specRef.current = spec;

  const loadResource = useCallback(async (forceRefresh: boolean) => {
    const id = ++requestIdRef.current;
    setIsLoading(true);

    await runResourceLoad(specRef.current, forceRefresh, (update) => {
      if (id !== requestIdRef.current) return;
      if (update.data !== undefined) setData(update.data);
      if (update.errorMessage !== undefined)
        setErrorMessage(update.errorMessage);
      if (update.isLoading !== undefined) setIsLoading(update.isLoading);
    });
  }, []);

  useEffect(() => {
    void loadResource(false);
  }, [loadResource]);

  const reload = useCallback(() => {
    void loadResource(true);
  }, [loadResource]);

  return { data, isLoading, errorMessage, reload };
}
