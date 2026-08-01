import { describe, expect, it } from "bun:test";
import { createTtlCache } from "./ttl-cache";

function countingCache(
  results: (() => Promise<string[]>)[],
  fallback?: () => Promise<string[]>,
) {
  let calls = 0;
  const cache = createTtlCache<string>({
    ttlMs: 30_000,
    failureMessage: "Laden fehlgeschlagen",
    unknownErrorMessage: "Unbekannter Fehler",
    load: () => {
      const result = results[Math.min(calls, results.length - 1)];
      calls += 1;
      return result();
    },
    fallback,
  });
  return { cache, callCount: () => calls };
}

const ok =
  (...data: string[]) =>
  async () =>
    data;
const fails = (message: string) => async () => {
  throw new Error(message);
};

describe("createTtlCache", () => {
  it("serves from cache within the ttl without a second load", async () => {
    const { cache, callCount } = countingCache([ok("a")]);

    const first = await cache.get({ nowMs: 1_000 });
    const second = await cache.get({ nowMs: 25_000 });

    expect(first.source).toBe("network");
    expect(second.source).toBe("cache");
    expect(second.data).toEqual(["a"]);
    expect(callCount()).toBe(1);
  });

  it("loads again once the ttl has expired", async () => {
    const { cache, callCount } = countingCache([ok("alt"), ok("neu")]);

    const first = await cache.get({ nowMs: 1_000 });
    const second = await cache.get({ nowMs: 31_500 });

    expect(first.data).toEqual(["alt"]);
    expect(second.data).toEqual(["neu"]);
    expect(callCount()).toBe(2);
  });

  it("loads again when a refresh is forced inside the ttl", async () => {
    const { cache, callCount } = countingCache([ok("alt"), ok("neu")]);

    await cache.get({ nowMs: 1_000 });
    const forced = await cache.get({ nowMs: 2_000, forceRefresh: true });

    expect(forced.data).toEqual(["neu"]);
    expect(callCount()).toBe(2);
  });

  it("does not let a forced refresh adopt an already running load", async () => {
    let releaseFirst: (value: string[]) => void = () => {};
    const pending = new Promise<string[]>((resolve) => {
      releaseFirst = resolve;
    });
    const { cache, callCount } = countingCache([() => pending, ok("neu")]);

    const joined = cache.get({ nowMs: 1_000 });
    const forced = cache.get({ nowMs: 1_000, forceRefresh: true });
    expect(callCount()).toBe(2);

    releaseFirst(["alt"]);

    expect((await forced).data).toEqual(["neu"]);
    expect((await joined).data).toEqual(["alt"]);
  });

  it("keeps the forced result when the superseded load settles last", async () => {
    let releaseFirst: (value: string[]) => void = () => {};
    const pending = new Promise<string[]>((resolve) => {
      releaseFirst = resolve;
    });
    const { cache } = countingCache([() => pending, ok("neu")]);

    const joined = cache.get({ nowMs: 1_000 });
    await cache.get({ nowMs: 1_000, forceRefresh: true });

    releaseFirst(["alt"]);
    await joined;

    expect((await cache.get({ nowMs: 1_500 })).data).toEqual(["neu"]);
  });

  it("coalesces parallel reads into a single load", async () => {
    let release: (value: string[]) => void = () => {};
    const pending = new Promise<string[]>((resolve) => {
      release = resolve;
    });
    const { cache, callCount } = countingCache([() => pending]);

    const both = Promise.all([
      cache.get({ nowMs: 2_000 }),
      cache.get({ nowMs: 2_000 }),
    ]);
    expect(callCount()).toBe(1);

    release(["parallel"]);
    const [first, second] = await both;

    expect(first.data).toEqual(second.data);
    expect(first.source).toBe("network");
    expect(second.source).toBe("network");
  });

  it("serves the stale entry with the error message when a reload fails", async () => {
    const { cache } = countingCache([ok("stabil"), fails("Backend weg")]);

    await cache.get({ nowMs: 1_000 });
    const stale = await cache.get({ nowMs: 45_000 });

    expect(stale.source).toBe("stale-cache");
    expect(stale.data).toEqual(["stabil"]);
    expect(stale.errorMessage).toBe("Backend weg");
  });

  it("falls back to the disk cache when the load fails with nothing in memory", async () => {
    const { cache } = countingCache([fails("Backend weg")], ok("von-platte"));

    const result = await cache.get({ nowMs: 1_000 });

    expect(result.source).toBe("disk-cache");
    expect(result.data).toEqual(["von-platte"]);
    expect(result.errorMessage).toBe("Backend weg");
  });

  it("reports the load failure when the disk fallback also fails", async () => {
    const { cache } = countingCache([fails("Backend weg")], async () => {
      throw new Error("Platte weg");
    });

    await expect(cache.get({ nowMs: 1_000 })).rejects.toThrow(
      "Laden fehlgeschlagen: Backend weg",
    );
  });

  it("keeps serving the disk fallback from memory afterwards", async () => {
    const { cache, callCount } = countingCache(
      [fails("Backend weg")],
      ok("von-platte"),
    );

    await cache.get({ nowMs: 1_000 });
    const cached = await cache.get({ nowMs: 1_500 });

    expect(cached.source).toBe("cache");
    expect(callCount()).toBe(1);
  });

  it("throws with the failure prefix when nothing can serve the read", async () => {
    const { cache } = countingCache([fails("Backend weg")]);

    await expect(cache.get({ nowMs: 1_000 })).rejects.toThrow(
      "Laden fehlgeschlagen: Backend weg",
    );
  });

  it("reports a non-Error rejection with the German fallback message", async () => {
    const { cache } = countingCache([
      async () => {
        throw "kaputt";
      },
    ]);

    await expect(cache.get({ nowMs: 1_000 })).rejects.toThrow(
      "Laden fehlgeschlagen: Unbekannter Fehler",
    );
  });

  it("rewrites cached entries in place without resetting the ttl", async () => {
    const { cache, callCount } = countingCache([ok("a", "b")]);
    await cache.get({ nowMs: 1_000 });

    cache.update((current) => current.filter((entry) => entry !== "a"));
    const after = await cache.get({ nowMs: 25_000 });

    expect(after.source).toBe("cache");
    expect(after.data).toEqual(["b"]);
    expect(callCount()).toBe(1);
  });

  it("ignores an update while nothing is cached", async () => {
    const { cache } = countingCache([ok("a")]);

    cache.update(() => ["ignoriert"]);

    expect((await cache.get({ nowMs: 1_000 })).data).toEqual(["a"]);
  });

  it("reset forces the next read back to the network", async () => {
    const { cache, callCount } = countingCache([ok("alt"), ok("neu")]);
    await cache.get({ nowMs: 1_000 });

    cache.reset();
    const afterReset = await cache.get({ nowMs: 2_000 });

    expect(afterReset.source).toBe("network");
    expect(afterReset.data).toEqual(["neu"]);
    expect(callCount()).toBe(2);
  });
});
