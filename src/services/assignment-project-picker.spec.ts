import { beforeEach, describe, expect, it, mock } from "bun:test";
import {
  loadProjectsForAssignmentPicker,
  test_resetAssignmentProjectPickerCache,
} from "./assignment-project-picker";

const mockDayliteSearchProjects = mock(() => Promise.resolve({} as unknown));

mock.module("../generated/tauri", () => ({
  commands: {
    dayliteSearchProjects: mockDayliteSearchProjects,
  },
}));

describe("assignment project picker service", () => {
  beforeEach(() => {
    mockDayliteSearchProjects.mockClear();
    test_resetAssignmentProjectPickerCache();
  });

  it("returns only new_status and in_progress projects via status-filtered search", async () => {
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

    const result = await loadProjectsForAssignmentPicker();

    expect(result).toHaveLength(2);
    expect(result[0].self).toBe("/v1/projects/1");
    expect(result[1].self).toBe("/v1/projects/2");
  });

  it("sends search with new_status and in_progress status filter", async () => {
    mockDayliteSearchProjects.mockResolvedValue({
      status: "ok",
      data: { results: [], next: null },
    });

    await loadProjectsForAssignmentPicker();

    expect(mockDayliteSearchProjects).toHaveBeenCalledTimes(1);
    const [input] = mockDayliteSearchProjects.mock.calls[0];
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

    await expect(loadProjectsForAssignmentPicker()).rejects.toThrow(
      "Serverfehler",
    );
  });
});
