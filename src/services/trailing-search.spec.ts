import { describe, expect, it } from "bun:test";
import type { DayliteProjectSummary } from "../generated/tauri";
import { createTrailingSearch } from "./trailing-search";

const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

function project(name: string): DayliteProjectSummary {
  return { self: `/v1/projects/${name}`, name, status: "in_progress" };
}

describe("createTrailingSearch", () => {
  it("queries only once on the settled term after the debounce delay", async () => {
    const calls: string[] = [];
    const results: DayliteProjectSummary[][] = [];
    const search = createTrailingSearch({
      delayMs: 20,
      minLength: 3,
      search: async (term) => {
        calls.push(term);
        return [project(term)];
      },
      onResults: (next) => results.push(next),
      onError: () => {},
    });

    search.setFilter("Pro");
    search.setFilter("Proj");
    search.setFilter("Proje");
    await wait(40);

    expect(calls).toEqual(["Proje"]);
    expect(results[results.length - 1]).toEqual([project("Proje")]);
  });

  it("clears results and sends no query below the minimum length", async () => {
    const calls: string[] = [];
    const results: DayliteProjectSummary[][] = [];
    const search = createTrailingSearch({
      delayMs: 10,
      minLength: 3,
      search: async (term) => {
        calls.push(term);
        return [project(term)];
      },
      onResults: (next) => results.push(next),
      onError: () => {},
    });

    search.setFilter("Pr");
    await wait(20);

    expect(calls).toEqual([]);
    expect(results).toEqual([[]]);
  });

  it("drops a stale response that resolves after a newer one", async () => {
    const resolvers = new Map<
      string,
      (value: DayliteProjectSummary[]) => void
    >();
    const results: DayliteProjectSummary[][] = [];
    const search = createTrailingSearch({
      delayMs: 5,
      minLength: 3,
      search: (term) =>
        new Promise((resolve) => {
          resolvers.set(term, resolve);
        }),
      onResults: (next) => results.push(next),
      onError: () => {},
    });

    search.setFilter("alt");
    await wait(10);
    search.setFilter("neu");
    await wait(10);

    // The newer request resolves first, then the stale one resolves late.
    resolvers.get("neu")?.([project("neu")]);
    await wait(0);
    resolvers.get("alt")?.([project("alt")]);
    await wait(0);

    expect(results).toEqual([[project("neu")]]);
  });

  it("invalidates an in-flight query when the filter drops below the minimum", async () => {
    const resolvers = new Map<
      string,
      (value: DayliteProjectSummary[]) => void
    >();
    const results: DayliteProjectSummary[][] = [];
    const search = createTrailingSearch({
      delayMs: 5,
      minLength: 3,
      search: (term) =>
        new Promise((resolve) => {
          resolvers.set(term, resolve);
        }),
      onResults: (next) => results.push(next),
      onError: () => {},
    });

    search.setFilter("abc");
    await wait(10);
    search.setFilter("ab");
    await wait(0);

    // The earlier request resolves late, but it has been invalidated.
    resolvers.get("abc")?.([project("abc")]);
    await wait(0);

    expect(results).toEqual([[]]);
  });
});
