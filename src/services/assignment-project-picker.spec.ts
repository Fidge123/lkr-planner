import { beforeEach, describe, expect, it, mock } from "bun:test";
import type { DayliteSearchInput } from "../generated/tauri";
import { mockCommands } from "../test/mock-commands";
import { searchProjectsForAssignmentPicker } from "./assignment-project-picker";

const mockDayliteSearchProjects = mock((_: DayliteSearchInput) =>
  Promise.resolve({} as unknown),
);

mockCommands({
  dayliteSearchProjects: mockDayliteSearchProjects,
});

describe("assignment project picker service", () => {
  beforeEach(() => {
    mockDayliteSearchProjects.mockClear();
  });

  it("fetches a candidate pool sorted by name, filtered to new_status and in_progress", async () => {
    mockDayliteSearchProjects.mockResolvedValue({
      status: "ok",
      data: { results: [], next: null },
    });

    await searchProjectsForAssignmentPicker("Nord");

    expect(mockDayliteSearchProjects).toHaveBeenCalledTimes(1);
    const [input] = mockDayliteSearchProjects.mock.calls[0];
    expect(input.searchTerm).toBe("Nord");
    expect(input.limit).toBe(50);
    expect(input.sort).toBe("name");
    expect(input.statuses).toContain("new_status");
    expect(input.statuses).toContain("in_progress");
  });

  it("shows at most 5 of the name-sorted candidates", async () => {
    mockDayliteSearchProjects.mockResolvedValue({
      status: "ok",
      data: {
        results: Array.from({ length: 8 }, (_, index) => ({
          self: `/v1/projects/${index}`,
          name: `Projekt ${index}`,
          status: "in_progress",
        })),
        next: null,
      },
    });

    const result = await searchProjectsForAssignmentPicker("Projekt");

    expect(result).toHaveLength(5);
    expect(result[0].self).toBe("/v1/projects/0");
    expect(result[4].self).toBe("/v1/projects/4");
  });
});
