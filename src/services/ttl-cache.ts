export type CacheSource = "network" | "cache" | "disk-cache" | "stale-cache";

export interface CacheLoadResult<T> {
  data: T[];
  source: CacheSource;
  errorMessage?: string | null;
}

export interface CacheLoadOptions {
  forceRefresh?: boolean;
}

interface TtlCacheOptions<T> {
  ttlMs: number;
  load: () => Promise<T[]>;
  /** Prefix of the error thrown when neither cache nor fallback can serve. */
  failureMessage: string;
  /** Consulted only when the load fails and nothing is held in memory. */
  fallback?: () => Promise<T[]>;
  /** Injectable so tests can control freshness without waiting. */
  now?: () => number;
}

export interface TtlCache<T> {
  get: (options?: CacheLoadOptions) => Promise<CacheLoadResult<T>>;
  /** Stores known data without counting as a fetch, sparing the fallback a second read. */
  seed: (data: T[]) => void;
  reset: () => void;
}

/** Concurrent callers share one in-flight request, so a burst costs one round trip. */
export function createTtlCache<T>({
  ttlMs,
  load,
  failureMessage,
  fallback,
  now = Date.now,
}: TtlCacheOptions<T>): TtlCache<T> {
  let entry: { data: T[]; fetchedAtMs: number } | null = null;
  let inFlight: Promise<CacheLoadResult<T>> | null = null;
  // Bumped per request and on reset, so a settling chain cannot overwrite a newer result.
  let generation = 0;

  function startLoad(): Promise<CacheLoadResult<T>> {
    const requestGeneration = ++generation;
    const isCurrent = () => requestGeneration === generation;

    return load()
      .then((data) => {
        if (isCurrent()) {
          entry = { data, fetchedAtMs: now() };
        }
        return { data, source: "network" } satisfies CacheLoadResult<T>;
      })
      .catch(async (error) => {
        const errorMessage = readErrorMessage(error);

        if (entry) {
          return {
            data: entry.data,
            source: "stale-cache",
            errorMessage,
          } satisfies CacheLoadResult<T>;
        }

        // A failing fallback must not mask the load error, nor reject unwrapped.
        const fromDisk = (await fallback?.().catch(() => [])) ?? [];
        if (fromDisk.length > 0) {
          if (isCurrent()) {
            entry = { data: fromDisk, fetchedAtMs: now() };
          }
          return {
            data: fromDisk,
            source: "disk-cache",
            errorMessage,
          } satisfies CacheLoadResult<T>;
        }

        throw new Error(`${failureMessage}: ${errorMessage}`);
      })
      .finally(() => {
        if (isCurrent()) {
          inFlight = null;
        }
      });
  }

  return {
    async get({
      forceRefresh = false,
    }: CacheLoadOptions = {}): Promise<CacheLoadResult<T>> {
      if (!forceRefresh && entry && now() - entry.fetchedAtMs < ttlMs) {
        return { data: entry.data, source: "cache" };
      }

      // A forced refresh must not adopt a request already running.
      if (forceRefresh) {
        inFlight = null;
      }

      inFlight ??= startLoad();

      return inFlight;
    },

    seed(data: T[]): void {
      if (entry || data.length === 0) return;
      // Stamped expired, so the pre-load seed does not suppress the network load.
      entry = { data, fetchedAtMs: now() - ttlMs };
    },

    reset(): void {
      // Supersedes a settling chain: its `then` would repopulate the cleared entry.
      // Its `finally` would also clear a later request's slot.
      generation += 1;
      entry = null;
      inFlight = null;
    },
  };
}

/**
 * Unreachable today: callCommand always throws an Error with a message.
 * Returns String(error) because a future loader may reject with other values.
 */
function readErrorMessage(error: unknown): string {
  if (error instanceof Error && error.message.trim().length > 0) {
    return error.message;
  }

  return String(error);
}
