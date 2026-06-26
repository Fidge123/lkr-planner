import { beforeEach, describe, expect, it, mock } from "bun:test";
import type { DayliteSearchInput } from "../generated/tauri";
import { searchProjectsForAssignmentPicker } from "./assignment-project-picker";

const mockDayliteSearchProjects = mock((_: DayliteSearchInput) =>
  Promise.resolve({} as unknown),
);

mock.module("../generated/tauri", () => ({
  commands: {
    dayliteSearchProjects: mockDayliteSearchProjects,
  },
}));

describe("assignment project picker service", () => {
  beforeEach(() => {
    mockDayliteSearchProjects.mockClear();
  });

  it("returns the projects from a successful search", async () => {
    mockDayliteSearchProjects.mockResolvedValue({
      status: "ok",
      data: {
        results: [
          {
            self: "/v1/projects/1",
            name: "Aktives Projekt",
            status: "in_progress",
          },
          {
            self: "/v1/projects/2",
            name: "Neues Projekt",
            status: "new_status",
          },
        ],
        next: null,
      },
    });

    const result = await searchProjectsForAssignmentPicker("Projekt");

    expect(result).toHaveLength(2);
    expect(result[0].self).toBe("/v1/projects/1");
    expect(result[1].self).toBe("/v1/projects/2");
  });

  it("requests max 5 results, sorted by name, filtered to new_status and in_progress", async () => {
    mockDayliteSearchProjects.mockResolvedValue({
      status: "ok",
      data: { results: [], next: null },
    });

    await searchProjectsForAssignmentPicker("Nord");

    expect(mockDayliteSearchProjects).toHaveBeenCalledTimes(1);
    const [input] = mockDayliteSearchProjects.mock.calls[0];
    expect(input.searchTerm).toBe("Nord");
    expect(input.limit).toBe(5);
    expect(input.sort).toBe("name");
    expect(input.statuses).toContain("new_status");
    expect(input.statuses).toContain("in_progress");
  });

  it("throws a German error message on API failure", async () => {
    mockDayliteSearchProjects.mockResolvedValue({
      status: "error",
      error: {
        code: "SERVER_ERROR",
        httpStatus: 500,
        userMessage: "Serverfehler",
        technicalMessage: "internal server error",
      },
    });

    await expect(searchProjectsForAssignmentPicker("Nord")).rejects.toThrow(
      "Serverfehler",
    );
  });
});
