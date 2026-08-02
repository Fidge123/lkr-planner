import { describe, expect, it } from "bun:test";
import type { CacheLoadResult } from "../../services/ttl-cache";
import { type ResourceUpdate, runResourceLoad } from "./use-cached-resource";

const loaded = (...data: string[]): CacheLoadResult<string> => ({
  data,
  source: "network",
});

function collect() {
  const updates: ResourceUpdate<string>[] = [];
  return {
    updates,
    emit: (update: ResourceUpdate<string>) => updates.push(update),
  };
}

const rejects = (message: string) => async () => {
  throw new Error(message);
};

describe("runResourceLoad", () => {
  it("emits the loaded data and clears the error", async () => {
    const { updates, emit } = collect();

    await runResourceLoad(
      { load: async () => loaded("a"), errorMessage: "Fehlgeschlagen" },
      false,
      emit,
    );

    expect(updates).toEqual([
      { data: ["a"], errorMessage: null },
      { isLoading: false },
    ]);
  });

  it("paints the preloaded data before the load resolves", async () => {
    const { updates, emit } = collect();

    await runResourceLoad(
      {
        load: async () => loaded("frisch"),
        errorMessage: "Fehlgeschlagen",
        preload: async () => ["von-platte"],
      },
      false,
      emit,
    );

    expect(updates[0]).toEqual({ data: ["von-platte"] });
    expect(updates[1]).toEqual({ data: ["frisch"], errorMessage: null });
  });

  it("skips the preload on a forced refresh", async () => {
    const { updates, emit } = collect();

    await runResourceLoad(
      {
        load: async () => loaded("frisch"),
        errorMessage: "Fehlgeschlagen",
        preload: async () => ["von-platte"],
      },
      true,
      emit,
    );

    expect(updates[0]).toEqual({ data: ["frisch"], errorMessage: null });
  });

  it("emits no data for an empty preload", async () => {
    const { updates, emit } = collect();

    await runResourceLoad(
      {
        load: async () => loaded("frisch"),
        errorMessage: "Fehlgeschlagen",
        preload: async () => [],
      },
      false,
      emit,
    );

    expect(updates[0]).toEqual({ data: ["frisch"], errorMessage: null });
  });

  it("surfaces a load rejection as its message and stops loading", async () => {
    const { updates, emit } = collect();

    await runResourceLoad(
      { load: rejects("Backend weg"), errorMessage: "Fehlgeschlagen" },
      false,
      emit,
    );

    expect(updates).toEqual([
      { errorMessage: "Backend weg" },
      { isLoading: false },
    ]);
  });

  it("falls back to the German message for a rejection without one", async () => {
    const { updates, emit } = collect();

    await runResourceLoad(
      {
        load: async () => {
          throw "kaputt";
        },
        errorMessage: "Fehlgeschlagen",
      },
      false,
      emit,
    );

    expect(updates[0]).toEqual({ errorMessage: "Fehlgeschlagen" });
  });

  it("still loads and still stops loading when the preload rejects", async () => {
    const { updates, emit } = collect();

    await runResourceLoad(
      {
        load: async () => loaded("frisch"),
        errorMessage: "Fehlgeschlagen",
        preload: rejects("Platte weg"),
      },
      false,
      emit,
    );

    expect(updates).toEqual([
      { data: ["frisch"], errorMessage: null },
      { isLoading: false },
    ]);
  });

  it("reports the load failure, not the preload failure, when both fail", async () => {
    const { updates, emit } = collect();

    await runResourceLoad(
      {
        load: rejects("Backend weg"),
        errorMessage: "Fehlgeschlagen",
        preload: rejects("Platte weg"),
      },
      false,
      emit,
    );

    expect(updates).toEqual([
      { errorMessage: "Backend weg" },
      { isLoading: false },
    ]);
  });
});
