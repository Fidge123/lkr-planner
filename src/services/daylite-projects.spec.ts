import { beforeEach, describe, expect, it, mock } from "bun:test";
import {
  loadDayliteProjects,
  test_resetDayliteProjectCache,
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
    test_resetDayliteProjectCache();
  });

  it("passes the Daylite list through unfiltered and unmapped", async () => {
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
        {
          self: "/v1/projects/7103",
          name: "Projekt Erledigt",
          status: "done",
          keywords: ["Aufträge"],
          category: "Liefertermin bekannt",
        },
      ],
    });

    const result = await loadDayliteProjects({ nowMs: 1_000 });

    expect(result.source).toBe("network");
    expect(result.errorMessage).toBeUndefined();
    expect(result.data.map((project) => project.self)).toEqual([
      "/v1/projects/7000",
      "/v1/projects/7103",
    ]);
    expect(result.data[0]).toEqual(
      expect.objectContaining({
        name: "Projekt Nord",
        status: "new_status",
        due: "2026-02-15T00:00:00.000Z",
        create_date: null,
        modify_date: "2026-02-15T11:45:00.000Z",
      }),
    );
  });

  it("throws the German command error when the backend fails", async () => {
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
