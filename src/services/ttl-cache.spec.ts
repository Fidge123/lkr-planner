import { describe, expect, it } from "bun:test";
import { createTtlCache } from "./ttl-cache";

function countingCache(
  results: (() => Promise<string[]>)[],
  fallback?: () => Promise<string[]>,
) {
  let calls = 0;
  let nowMs = 1_000;
  const cache = createTtlCache<string>({
    ttlMs: 30_000,
    failureMessage: "Laden fehlgeschlagen",
    unknownErrorMessage: "Unbekannter Fehler",
    now: () => nowMs,
    load: () => {
      const result = results[Math.min(calls, results.length - 1)];
      calls += 1;
      return result();
    },
    fallback,
  });
  return {
    cache,
    callCount: () => calls,
    setNow: (ms: number) => {
      nowMs = ms;
    },
  };
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
    const { cache, callCount, setNow } = countingCache([ok("a")]);

    const first = await cache.get();
    setNow(25_000);
    const second = await cache.get();

    expect(first.source).toBe("network");
    expect(second.source).toBe("cache");
    expect(second.data).toEqual(["a"]);
    expect(callCount()).toBe(1);
  });

  it("loads again once the ttl has expired", async () => {
    const { cache, callCount, setNow } = countingCache([ok("alt"), ok("neu")]);

    const first = await cache.get();
    setNow(31_500);
    const second = await cache.get();

    expect(first.data).toEqual(["alt"]);
    expect(second.data).toEqual(["neu"]);
    expect(callCount()).toBe(2);
  });

  it("ages the entry from when the data arrived, not when it was requested", async () => {
    let release: (value: string[]) => void = () => {};
    const pending = new Promise<string[]>((resolve) => {
      release = resolve;
    });
    const { cache, callCount, setNow } = countingCache([
      () => pending,
      ok("neu"),
    ]);

    const slow = cache.get();
    setNow(40_000); // the load outlived the 30s ttl
    release(["alt"]);
    await slow;

    setNow(50_000); // only 10s after the data actually landed
    const second = await cache.get();

    expect(second.source).toBe("cache");
    expect(second.data).toEqual(["alt"]);
    expect(callCount()).toBe(1);
  });

  it("loads again when a refresh is forced inside the ttl", async () => {
    const { cache, callCount } = countingCache([ok("alt"), ok("neu")]);

    await cache.get();
    const forced = await cache.get({ forceRefresh: true });

    expect(forced.data).toEqual(["neu"]);
    expect(callCount()).toBe(2);
  });

  it("does not let a forced refresh adopt an already running load", async () => {
    let releaseFirst: (value: string[]) => void = () => {};
    const pending = new Promise<string[]>((resolve) => {
      releaseFirst = resolve;
    });
    const { cache, callCount } = countingCache([() => pending, ok("neu")]);

    const joined = cache.get();
    const forced = cache.get({ forceRefresh: true });
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

    const joined = cache.get();
    await cache.get({ forceRefresh: true });

    releaseFirst(["alt"]);
    await joined;

    expect((await cache.get()).data).toEqual(["neu"]);
  });

  it("coalesces parallel reads into a single load", async () => {
    let release: (value: string[]) => void = () => {};
    const pending = new Promise<string[]>((resolve) => {
      release = resolve;
    });
    const { cache, callCount } = countingCache([() => pending]);

    const both = Promise.all([cache.get(), cache.get()]);
    expect(callCount()).toBe(1);

    release(["parallel"]);
    const [first, second] = await both;

    expect(first.data).toEqual(second.data);
    expect(first.source).toBe("network");
    expect(second.source).toBe("network");
  });

  it("serves the stale entry with the error message when a reload fails", async () => {
    const { cache, setNow } = countingCache([
      ok("stabil"),
      fails("Backend weg"),
    ]);

    await cache.get();
    setNow(45_000);
    const stale = await cache.get();

    expect(stale.source).toBe("stale-cache");
    expect(stale.data).toEqual(["stabil"]);
    expect(stale.errorMessage).toBe("Backend weg");
  });

  it("falls back to the disk cache when the load fails with nothing in memory", async () => {
    const { cache } = countingCache([fails("Backend weg")], ok("von-platte"));

    const result = await cache.get();

    expect(result.source).toBe("disk-cache");
    expect(result.data).toEqual(["von-platte"]);
    expect(result.errorMessage).toBe("Backend weg");
  });

  it("reports the load failure when the disk fallback also fails", async () => {
    const { cache } = countingCache([fails("Backend weg")], async () => {
      throw new Error("Platte weg");
    });

    await expect(cache.get()).rejects.toThrow(
      "Laden fehlgeschlagen: Backend weg",
    );
  });

  it("keeps serving the disk fallback from memory afterwards", async () => {
    const { cache, callCount } = countingCache(
      [fails("Backend weg")],
      ok("von-platte"),
    );

    await cache.get();
    const cached = await cache.get();

    expect(cached.source).toBe("cache");
    expect(callCount()).toBe(1);
  });

  it("throws with the failure prefix when nothing can serve the read", async () => {
    const { cache } = countingCache([fails("Backend weg")]);

    await expect(cache.get()).rejects.toThrow(
      "Laden fehlgeschlagen: Backend weg",
    );
  });

  it("reports a non-Error rejection with the German fallback message", async () => {
    const { cache } = countingCache([
      async () => {
        throw "kaputt";
      },
    ]);

    await expect(cache.get()).rejects.toThrow(
      "Laden fehlgeschlagen: Unbekannter Fehler",
    );
  });

  it("rewrites cached entries in place without resetting the ttl", async () => {
    const { cache, callCount, setNow } = countingCache([ok("a", "b")]);
    await cache.get();

    cache.update((current) => current.filter((entry) => entry !== "a"));
    setNow(25_000);
    const after = await cache.get();

    expect(after.source).toBe("cache");
    expect(after.data).toEqual(["b"]);
    expect(callCount()).toBe(1);
  });

  it("ignores an update while nothing is cached", async () => {
    const { cache } = countingCache([ok("a")]);

    cache.update(() => ["ignoriert"]);

    expect((await cache.get()).data).toEqual(["a"]);
  });

  it("reset forces the next read back to the network", async () => {
    const { cache, callCount } = countingCache([ok("alt"), ok("neu")]);
    await cache.get();

    cache.reset();
    const afterReset = await cache.get();

    expect(afterReset.source).toBe("network");
    expect(afterReset.data).toEqual(["neu"]);
    expect(callCount()).toBe(2);
  });
});
