import { beforeEach, describe, expect, it, mock } from "bun:test";
import {
  DEFAULT_DAYLITE_PROJECT_CACHE_TTL_MS,
  loadDayliteProjects,
  resetDayliteProjectCacheForTests,
  setDayliteProjectCacheTtlMs,
} from "./daylite-projects";

const mockDayliteListProjects = mock(() => Promise.resolve({} as unknown));

mock.module("../generated/tauri", () => ({
  commands: {
    dayliteListProjects: mockDayliteListProjects,
  },
}));

describe("daylite project service", () => {
  beforeEach(() => {
    mockDayliteListProjects.mockClear();
    resetDayliteProjectCacheForTests();
    setDayliteProjectCacheTtlMs(DEFAULT_DAYLITE_PROJECT_CACHE_TTL_MS);
  });

  it("maps and normalizes project status/date fields", async () => {
    mockDayliteListProjects.mockResolvedValue({
      status: "ok",
      data: [
        {
          reference: "/v1/projects/7000",
          name: "Projekt Nord",
          status: "NEW",
          due: "2026-02-15",
          createDate: "not-a-date",
          modifyDate: "2026-02-15T12:45:00+01:00",
        },
      ],
    });

    const result = await loadDayliteProjects({ nowMs: 1_000 });

    expect(result.source).toBe("network");
    expect(result.errorMessage).toBeNull();
    expect(result.projects).toEqual([
      {
        self: "/v1/projects/7000",
        name: "Projekt Nord",
        status: "new_status",
        due: "2026-02-15T00:00:00.000Z",
        modify_date: "2026-02-15T11:45:00.000Z",
      },
    ]);
  });

  it("reuses cached data within the default 30s ttl", async () => {
    mockDayliteListProjects.mockResolvedValue({
      status: "ok",
      data: [
        {
          reference: "/v1/projects/7001",
          name: "Projekt Ost",
          status: "in_progress",
        },
      ],
    });

    const first = await loadDayliteProjects({ nowMs: 1_000 });
    const second = await loadDayliteProjects({ nowMs: 25_000 });

    expect(first.source).toBe("network");
    expect(second.source).toBe("cache");
    expect(mockDayliteListProjects).toHaveBeenCalledTimes(1);
  });

  it("does not increase request count for secondary ui consumers (overview section)", async () => {
    mockDayliteListProjects.mockResolvedValue({
      status: "ok",
      data: [
        {
          reference: "/v1/projects/7010",
          name: "Projekt Mehrfachnutzung",
          status: "in_progress",
        },
      ],
    });

    const planningData = await loadDayliteProjects({ nowMs: 10_000 });
    const overviewData = await loadDayliteProjects({ nowMs: 10_010 });

    expect(planningData.projects[0]?.self).toBe("/v1/projects/7010");
    expect(overviewData.projects[0]?.self).toBe("/v1/projects/7010");
    expect(mockDayliteListProjects).toHaveBeenCalledTimes(1);
  });

  it("loads fresh data when ttl expires", async () => {
    mockDayliteListProjects
      .mockResolvedValueOnce({
        status: "ok",
        data: [
          {
            reference: "/v1/projects/7002",
            name: "Projekt Alt",
            status: "in_progress",
          },
        ],
      })
      .mockResolvedValueOnce({
        status: "ok",
        data: [
          {
            reference: "/v1/projects/7003",
            name: "Projekt Neu",
            status: "done",
          },
        ],
      });

    const first = await loadDayliteProjects({ nowMs: 1_000 });
    const second = await loadDayliteProjects({ nowMs: 31_500 });

    expect(first.projects[0]?.self).toBe("/v1/projects/7002");
    expect(second.projects[0]?.self).toBe("/v1/projects/7003");
    expect(mockDayliteListProjects).toHaveBeenCalledTimes(2);
  });

  it("coalesces parallel reads into a single backend request", async () => {
    let resolveCommand: (value: unknown) => void = () => {};
    const commandPromise = new Promise((resolve) => {
      resolveCommand = resolve;
    });

    mockDayliteListProjects.mockReturnValue(commandPromise);

    const firstPromise = loadDayliteProjects({ nowMs: 2_000 });
    const secondPromise = loadDayliteProjects({ nowMs: 2_000 });

    expect(mockDayliteListProjects).toHaveBeenCalledTimes(1);

    resolveCommand({
      status: "ok",
      data: [
        {
          reference: "/v1/projects/7004",
          name: "Projekt Parallel",
          status: "in_progress",
        },
      ],
    });

    const [first, second] = await Promise.all([firstPromise, secondPromise]);

    expect(first.projects).toEqual(second.projects);
    expect(first.source).toBe("network");
    expect(second.source).toBe("network");
  });

  it("returns stale cached data with german error message on transient fetch error", async () => {
    mockDayliteListProjects
      .mockResolvedValueOnce({
        status: "ok",
        data: [
          {
            reference: "/v1/projects/7005",
            name: "Projekt Stabil",
            status: "in_progress",
          },
        ],
      })
      .mockResolvedValueOnce({
        status: "error",
        error: {
          userMessage: "Die Daten konnten nicht von Daylite geladen werden.",
        },
      });

    await loadDayliteProjects({ nowMs: 1_000 });
    const staleFallback = await loadDayliteProjects({ nowMs: 45_000 });

    expect(staleFallback.source).toBe("stale-cache");
    expect(staleFallback.projects[0]?.self).toBe("/v1/projects/7005");
    expect(staleFallback.errorMessage).toBe(
      "Die Daten konnten nicht von Daylite geladen werden.",
    );
  });

  it("throws a german error when there is no cache fallback", async () => {
    mockDayliteListProjects.mockResolvedValue({
      status: "error",
      error: {
        userMessage: "Die Daten konnten nicht von Daylite geladen werden.",
      },
    });

    await expect(loadDayliteProjects({ nowMs: 1_000 })).rejects.toThrow(
      "Projektladen fehlgeschlagen: Die Daten konnten nicht von Daylite geladen werden.",
    );
  });
});
