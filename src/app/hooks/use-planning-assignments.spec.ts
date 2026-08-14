import { describe, expect, it } from "bun:test";
import { loadWeekWithPrefetch } from "./use-planning-assignments";

function deferred() {
  let resolve: () => void = () => {};
  const promise = new Promise<void>((resolvePromise) => {
    resolve = resolvePromise;
  });
  return { promise, resolve };
}

describe("loadWeekWithPrefetch", () => {
  it("holds the prefetch back until the active week has loaded", async () => {
    const active = deferred();
    const prefetched: string[] = [];

    const sequence = loadWeekWithPrefetch(
      "2026-08-10",
      () => active.promise,
      async (ws) => {
        prefetched.push(ws);
      },
      () => false,
    );

    await Promise.resolve();
    expect(prefetched).toEqual([]);

    active.resolve();
    await sequence;

    expect(prefetched.sort()).toEqual(["2026-08-03", "2026-08-17"]);
  });

  it("skips the prefetch when the week changed while the active load ran", async () => {
    const prefetched: string[] = [];

    await loadWeekWithPrefetch(
      "2026-08-10",
      async () => {},
      async (ws) => {
        prefetched.push(ws);
      },
      () => true,
    );

    expect(prefetched).toEqual([]);
  });

  it("still prefetches when the active week came from the cache", async () => {
    const prefetched: string[] = [];

    await loadWeekWithPrefetch(
      "2026-08-10",
      async () => {},
      async (ws) => {
        prefetched.push(ws);
      },
      () => false,
    );

    expect(prefetched.sort()).toEqual(["2026-08-03", "2026-08-17"]);
  });
});
