import { describe, expect, it } from "bun:test";
import {
  loadWeekWithPrefetch,
  type WeekLoadSequence,
} from "./use-planning-assignments";

function deferred() {
  let resolve: () => void = () => {};
  const promise = new Promise<void>((resolvePromise) => {
    resolve = resolvePromise;
  });
  return { promise, resolve };
}

function sequence(overrides: Partial<WeekLoadSequence> = {}) {
  const prefetched: string[] = [];
  const waited: number[] = [];

  const run = () =>
    loadWeekWithPrefetch({
      weekStart: "2026-08-10",
      loadActive: async () => {},
      prefetch: async (weekStart) => {
        prefetched.push(weekStart);
      },
      isCancelled: () => false,
      wait: async (ms) => {
        waited.push(ms);
      },
      ...overrides,
    });

  return { run, prefetched, waited };
}

describe("loadWeekWithPrefetch", () => {
  it("holds the prefetch back until the active week has loaded", async () => {
    const active = deferred();
    const { run, prefetched } = sequence({ loadActive: () => active.promise });

    const running = run();
    await Promise.resolve();
    expect(prefetched).toEqual([]);

    active.resolve();
    await running;

    expect(prefetched.sort()).toEqual(["2026-08-03", "2026-08-17"]);
  });

  it("waits out the idle window before dispatching the prefetch", async () => {
    const { run, prefetched, waited } = sequence({
      wait: async (ms) => {
        waited.push(ms);
        expect(prefetched).toEqual([]);
      },
    });

    await run();

    expect(waited).toHaveLength(1);
    expect(waited[0]).toBeGreaterThan(0);
    expect(prefetched.sort()).toEqual(["2026-08-03", "2026-08-17"]);
  });

  it("skips the prefetch when the week changed during the idle window", async () => {
    let cancelled = false;
    const { run, prefetched } = sequence({
      isCancelled: () => cancelled,
      wait: async () => {
        cancelled = true;
      },
    });

    await run();

    expect(prefetched).toEqual([]);
  });

  it("skips the idle window entirely when the week already changed", async () => {
    const { run, prefetched, waited } = sequence({ isCancelled: () => true });

    await run();

    expect(waited).toEqual([]);
    expect(prefetched).toEqual([]);
  });

  it("still prefetches when the active week came from the cache", async () => {
    const { run, prefetched } = sequence();

    await run();

    expect(prefetched.sort()).toEqual(["2026-08-03", "2026-08-17"]);
  });
});
