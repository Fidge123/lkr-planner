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
  /** Prefixes the error thrown when neither the cache nor the fallback can serve. */
  failureMessage: string;
  /** Reported when a rejection carries no message of its own. */
  unknownErrorMessage: string;
  /** Consulted only when the load fails and nothing is held in memory. */
  fallback?: () => Promise<T[]>;
  /** Injectable so tests can control freshness without waiting. */
  now?: () => number;
}

export interface TtlCache<T> {
  get: (options?: CacheLoadOptions) => Promise<CacheLoadResult<T>>;
  /** Rewrites the cached entries in place, keeping the current fetch timestamp. */
  update: (next: (current: T[]) => T[]) => void;
  reset: () => void;
}

/**
 * Concurrent callers share one in-flight request, so a screen that mounts several
 * consumers at once still costs a single backend round trip.
 */
export function createTtlCache<T>({
  ttlMs,
  load,
  failureMessage,
  unknownErrorMessage,
  fallback,
  now = Date.now,
}: TtlCacheOptions<T>): TtlCache<T> {
  let entry: { data: T[]; fetchedAtMs: number } | null = null;
  let inFlight: Promise<CacheLoadResult<T>> | null = null;
  // Bumped whenever a request is superseded, so a chain that is still settling
  // cannot write its result over the newer one that replaced it.
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
        const errorMessage = readErrorMessage(error, unknownErrorMessage);

        if (entry) {
          return {
            data: entry.data,
            source: "stale-cache",
            errorMessage,
          } satisfies CacheLoadResult<T>;
        }

        // A failing fallback must not replace the load failure the caller needs to
        // see, nor escape as a rejection this catch never wrapped.
        const fromFallback = (await fallback?.().catch(() => [])) ?? [];
        if (fromFallback.length > 0) {
          if (isCurrent()) {
            entry = { data: fromFallback, fetchedAtMs: now() };
          }
          return {
            data: fromFallback,
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

      // A forced refresh asks for data newer than now, so it must not adopt a
      // request that was already running when the caller asked.
      if (forceRefresh) {
        inFlight = null;
      }

      inFlight ??= startLoad();

      return inFlight;
    },

    update(next: (current: T[]) => T[]): void {
      if (!entry) return;
      entry = { data: next(entry.data), fetchedAtMs: entry.fetchedAtMs };
    },

    reset(): void {
      entry = null;
      inFlight = null;
    },
  };
}

function readErrorMessage(error: unknown, fallbackMessage: string): string {
  if (error instanceof Error && error.message.trim().length > 0) {
    return error.message;
  }

  return fallbackMessage;
}
