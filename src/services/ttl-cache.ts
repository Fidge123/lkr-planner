export type CacheSource = "network" | "cache" | "disk-cache" | "stale-cache";

export interface CacheLoadResult<T> {
  data: T[];
  source: CacheSource;
  errorMessage?: string | null;
}

export interface CacheLoadOptions {
  nowMs?: number;
  forceRefresh?: boolean;
}

interface TtlCacheOptions<T> {
  ttlMs: number;
  load: () => Promise<T[]>;
  /** Prefixes the error thrown when neither the cache nor the fallback can serve. */
  failureMessage: string;
  /** Consulted only when the load fails and nothing is held in memory. */
  fallback?: () => Promise<T[]>;
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
  fallback,
}: TtlCacheOptions<T>): TtlCache<T> {
  let entry: { data: T[]; fetchedAtMs: number } | null = null;
  let inFlight: Promise<CacheLoadResult<T>> | null = null;

  return {
    async get({
      nowMs = Date.now(),
      forceRefresh = false,
    }: CacheLoadOptions = {}): Promise<CacheLoadResult<T>> {
      if (!forceRefresh && entry && nowMs - entry.fetchedAtMs < ttlMs) {
        return { data: entry.data, source: "cache" };
      }

      inFlight ??= load()
        .then((data) => {
          entry = { data, fetchedAtMs: nowMs };
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

          const fromFallback = (await fallback?.()) ?? [];
          if (fromFallback.length > 0) {
            entry = { data: fromFallback, fetchedAtMs: nowMs };
            return {
              data: fromFallback,
              source: "disk-cache",
              errorMessage,
            } satisfies CacheLoadResult<T>;
          }

          throw new Error(`${failureMessage}: ${errorMessage}`);
        })
        .finally(() => {
          inFlight = null;
        });

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

function readErrorMessage(error: unknown): string {
  if (error instanceof Error && error.message.trim().length > 0) {
    return error.message;
  }

  return "Die Daten konnten nicht von Daylite geladen werden.";
}
