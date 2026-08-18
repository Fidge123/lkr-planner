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
  /** Consulted only when the load fails and nothing is held in memory. */
  fallback?: () => Promise<T[]>;
  /** Injectable so tests can control freshness without waiting. */
  now?: () => number;
}

export interface TtlCache<T> {
  get: (options?: CacheLoadOptions) => Promise<CacheLoadResult<T>>;
  /**
   * Publishes already-known data without counting as a fetch, so a caller that has
   * read it from elsewhere can spare the fallback a second read of the same source.
   */
  seed: (data: T[]) => void;
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
        const errorMessage = readErrorMessage(error);

        if (entry) {
          return {
            data: entry.data,
            source: "stale-cache",
            errorMessage,
          } satisfies CacheLoadResult<T>;
        }

        // A failing fallback must not replace the load failure the caller needs to
        // see, nor escape as a rejection this catch never wrapped.
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

      // A forced refresh asks for data newer than now, so it must not adopt a
      // request that was already running when the caller asked.
      if (forceRefresh) {
        inFlight = null;
      }

      inFlight ??= startLoad();

      return inFlight;
    },

    seed(data: T[]): void {
      if (entry || data.length === 0) return;
      // Stamped as already expired: unlike the fallback, which runs because the
      // network just failed, seeding happens before the load and must not suppress it.
      entry = { data, fetchedAtMs: now() - ttlMs };
    },

    reset(): void {
      // Supersedes any settling chain: without this its `then` would repopulate the
      // entry just cleared, and its `finally` would clear a later request's slot.
      generation += 1;
      entry = null;
      inFlight = null;
    },
  };
}

/**
 * Callers reach this cache through unwrapCommandResult, which always throws an Error
 * with a message, so the last resort is unreachable today. It keeps whatever the value
 * stringifies to rather than a fixed sentence, because if some future loader does
 * reject with something else, that detail is the only clue there will be.
 */
function readErrorMessage(error: unknown): string {
  if (error instanceof Error && error.message.trim().length > 0) {
    return error.message;
  }

  return String(error);
}
