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

  it("serves seeded data from memory instead of consulting the fallback", async () => {
    let fallbackCalls = 0;
    const { cache } = countingCache([fails("Backend weg")], async () => {
      fallbackCalls += 1;
      return ["von-platte"];
    });

    cache.seed(["vorab"]);
    const result = await cache.get();

    expect(result.source).toBe("stale-cache");
    expect(result.data).toEqual(["vorab"]);
    expect(fallbackCalls).toBe(0);
  });

  it("seeded data never counts as fresh, so the load still runs", async () => {
    const { cache, callCount } = countingCache([ok("frisch")]);

    cache.seed(["vorab"]);
    const result = await cache.get();

    expect(result.source).toBe("network");
    expect(result.data).toEqual(["frisch"]);
    expect(callCount()).toBe(1);
  });

  it("seeding does not displace data already held", async () => {
    const { cache } = countingCache([ok("frisch")]);
    await cache.get();

    cache.seed(["vorab"]);

    expect((await cache.get()).data).toEqual(["frisch"]);
  });

  it("throws with the failure prefix when nothing can serve the read", async () => {
    const { cache } = countingCache([fails("Backend weg")]);

    await expect(cache.get()).rejects.toThrow(
      "Laden fehlgeschlagen: Backend weg",
    );
  });

  it("keeps what a non-Error rejection stringifies to", async () => {
    const { cache } = countingCache([
      async () => {
        throw "kaputt";
      },
    ]);

    await expect(cache.get()).rejects.toThrow("Laden fehlgeschlagen: kaputt");
  });

  it("falls back to the stringified Error when it carries no message", async () => {
    const { cache } = countingCache([
      async () => {
        throw new Error("");
      },
    ]);

    await expect(cache.get()).rejects.toThrow(/^Laden fehlgeschlagen: Error$/);
  });

  it("reset stops a settling load from repopulating the cache", async () => {
    let release: (value: string[]) => void = () => {};
    const pending = new Promise<string[]>((resolve) => {
      release = resolve;
    });
    const { cache } = countingCache([() => pending, ok("neu")]);

    const abandoned = cache.get();
    cache.reset();
    release(["alt"]);
    await abandoned;

    expect((await cache.get()).data).toEqual(["neu"]);
  });

  it("reset stops a settling load from clearing a later request", async () => {
    let release: (value: string[]) => void = () => {};
    const pending = new Promise<string[]>((resolve) => {
      release = resolve;
    });
    const { cache, callCount } = countingCache([() => pending, ok("neu")]);

    const abandoned = cache.get();
    cache.reset();
    const second = cache.get();
    release(["alt"]);
    await abandoned;

    // The abandoned chain must not have freed the slot the second read owns.
    await cache.get();
    expect(callCount()).toBe(2);
    expect((await second).data).toEqual(["neu"]);
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
