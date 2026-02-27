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

  it("passes through planning-ready project status/date fields", async () => {
    mockDayliteListProjects.mockResolvedValue({
      status: "ok",
      data: [
        {
          self: "/v1/projects/7000",
          name: "Projekt Nord",
          status: "new_status",
          category: "Überfällig",
          due: "2026-02-15T00:00:00.000Z",
          create_date: null,
          modify_date: "2026-02-15T11:45:00.000Z",
          keywords: [],
        },
      ],
    });

    const result = await loadDayliteProjects({ nowMs: 1_000 });

    expect(result.source).toBe("network");
    expect(result.errorMessage).toBeNull();
    expect(result.projects).toEqual([
      expect.objectContaining({
        self: "/v1/projects/7000",
        name: "Projekt Nord",
        status: "new_status",
        due: "2026-02-15T00:00:00.000Z",
        create_date: null,
        modify_date: "2026-02-15T11:45:00.000Z",
      }),
    ]);
  });

  it("maps project self field from tauri command payload", async () => {
    mockDayliteListProjects.mockResolvedValue({
      status: "ok",
      data: [
        {
          self: "/v1/projects/7999",
          name: "Projekt Self Feld",
          status: "in_progress",
        },
      ],
    });

    const result = await loadDayliteProjects({ nowMs: 1_000 });

    expect(result.projects[0]?.self).toBe("/v1/projects/7999");
    expect(result.projects[0]?.name).toBe("Projekt Self Feld");
  });

  it("returns the unfiltered Daylite list after loading projects", async () => {
    mockDayliteListProjects.mockResolvedValue({
      status: "ok",
      data: [
        {
          self: "/v1/projects/7101",
          name: "Projekt Pipeline",
          status: "in_progress",
          keywords: ["Aufträge", "Vorbereitung"],
          category: "Neutral",
        },
        {
          self: "/v1/projects/7102",
          name: "Projekt Kategorie",
          status: "new_status",
          keywords: ["Sonstiges"],
          category: "Überfällig",
        },
        {
          self: "/v1/projects/7103",
          name: "Projekt Erledigt",
          status: "done",
          keywords: ["Aufträge", "Durchführung"],
          category: "Liefertermin bekannt",
        },
        {
          self: "/v1/projects/7104",
          name: "Projekt Ohne Treffer",
          status: "in_progress",
          keywords: ["Sonstiges"],
          category: "Neutral",
        },
      ],
    });

    const result = await loadDayliteProjects({ nowMs: 1_000 });

    expect(result.projects.map((project) => project.self)).toEqual([
      "/v1/projects/7101",
      "/v1/projects/7102",
      "/v1/projects/7103",
      "/v1/projects/7104",
    ]);
  });

  it("reuses cached data within the default 30s ttl", async () => {
    mockDayliteListProjects.mockResolvedValue({
      status: "ok",
      data: [
        {
          self: "/v1/projects/7001",
          name: "Projekt Ost",
          status: "in_progress",
          category: "Liefertermin bekannt",
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
          self: "/v1/projects/7010",
          name: "Projekt Mehrfachnutzung",
          status: "in_progress",
          keywords: ["Aufträge", "Vorbereitung"],
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
            self: "/v1/projects/7002",
            name: "Projekt Alt",
            status: "in_progress",
            keywords: ["Aufträge", "Vorbereitung"],
          },
        ],
      })
      .mockResolvedValueOnce({
        status: "ok",
        data: [
          {
            self: "/v1/projects/7003",
            name: "Projekt Neu",
            status: "in_progress",
            keywords: ["Aufträge", "Durchführung"],
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
          self: "/v1/projects/7004",
          name: "Projekt Parallel",
          status: "in_progress",
          category: "Liefertermin bekannt",
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
            self: "/v1/projects/7005",
            name: "Projekt Stabil",
            status: "in_progress",
            category: "Überfällig",
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
