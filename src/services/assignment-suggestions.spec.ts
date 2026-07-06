import { beforeEach, describe, expect, it, mock } from "bun:test";
import type { DayliteProjectSummary } from "../generated/tauri";
import {
  combineSuggestions,
  getLastAssignedProject,
  loadDefaultSuggestions,
  recordLastAssignedProject,
  resetLastAssignedProject,
} from "./assignment-suggestions";

const mockDayliteQueryOverdueProjects = mock(() =>
  Promise.resolve({} as unknown),
);

mock.module("../generated/tauri", () => ({
  commands: {
    dayliteQueryOverdueProjects: mockDayliteQueryOverdueProjects,
  },
}));

function project(name: string, ref: string): DayliteProjectSummary {
  return { self: ref, name };
}

describe("last-used cache", () => {
  beforeEach(() => {
    resetLastAssignedProject();
  });

  it("starts empty (no recent assignment this session)", () => {
    expect(getLastAssignedProject()).toBeNull();
  });

  it("returns the most recently recorded project", () => {
    recordLastAssignedProject(project("Projekt Alt", "/v1/projects/1"));
    recordLastAssignedProject(project("Projekt Neu", "/v1/projects/2"));

    expect(getLastAssignedProject()?.self).toBe("/v1/projects/2");
    expect(getLastAssignedProject()?.name).toBe("Projekt Neu");
  });
});

describe("combineSuggestions", () => {
  const overdue = [
    project("Projekt 10", "/v1/projects/10"),
    project("Projekt 11", "/v1/projects/11"),
    project("Projekt 12", "/v1/projects/12"),
    project("Projekt 13", "/v1/projects/13"),
    project("Projekt 14", "/v1/projects/14"),
  ];

  it("puts the recent project first, followed by overdue projects", () => {
    const recent = project("Projekt Zuletzt", "/v1/projects/99");

    const suggestions = combineSuggestions(recent, overdue);

    expect(suggestions[0].self).toBe("/v1/projects/99");
    expect(suggestions[1].self).toBe("/v1/projects/10");
  });

  it("caps the combined list at 5 suggestions", () => {
    const recent = project("Projekt Zuletzt", "/v1/projects/99");

    const suggestions = combineSuggestions(recent, overdue);

    expect(suggestions).toHaveLength(5);
    expect(suggestions.map((p) => p.self)).toEqual([
      "/v1/projects/99",
      "/v1/projects/10",
      "/v1/projects/11",
      "/v1/projects/12",
      "/v1/projects/13",
    ]);
  });

  it("keeps a recent project that is also overdue only in the first position", () => {
    const recent = project("Projekt 11", "/v1/projects/11");

    const suggestions = combineSuggestions(recent, overdue);

    expect(suggestions).toHaveLength(5);
    expect(suggestions[0].self).toBe("/v1/projects/11");
    const occurrences = suggestions.filter((p) => p.self === "/v1/projects/11");
    expect(occurrences).toHaveLength(1);
    // Dedup happens before the cap, so a 5th distinct project fills the list.
    expect(suggestions.map((p) => p.self)).toContain("/v1/projects/14");
  });

  it("shows up to 5 overdue projects when the cache is empty", () => {
    const suggestions = combineSuggestions(null, [
      ...overdue,
      project("Projekt 15", "/v1/projects/15"),
    ]);

    expect(suggestions).toHaveLength(5);
    expect(suggestions.map((p) => p.self)).toEqual([
      "/v1/projects/10",
      "/v1/projects/11",
      "/v1/projects/12",
      "/v1/projects/13",
      "/v1/projects/14",
    ]);
  });

  it("returns only the recent project when there are no overdue projects", () => {
    const recent = project("Projekt Zuletzt", "/v1/projects/99");

    expect(combineSuggestions(recent, [])).toEqual([recent]);
  });

  it("returns an empty list when neither recent nor overdue projects exist", () => {
    expect(combineSuggestions(null, [])).toEqual([]);
  });
});

describe("loadDefaultSuggestions", () => {
  beforeEach(() => {
    resetLastAssignedProject();
    mockDayliteQueryOverdueProjects.mockClear();
  });

  it("combines the cached recent project with the overdue query results", async () => {
    recordLastAssignedProject(project("Projekt Zuletzt", "/v1/projects/99"));
    mockDayliteQueryOverdueProjects.mockResolvedValue({
      status: "ok",
      data: [project("Projekt 10", "/v1/projects/10")],
    });

    const suggestions = await loadDefaultSuggestions();

    expect(suggestions.map((p) => p.self)).toEqual([
      "/v1/projects/99",
      "/v1/projects/10",
    ]);
  });

  it("returns only overdue projects when no recent assignment exists", async () => {
    mockDayliteQueryOverdueProjects.mockResolvedValue({
      status: "ok",
      data: [
        project("Projekt 10", "/v1/projects/10"),
        project("Projekt 11", "/v1/projects/11"),
      ],
    });

    const suggestions = await loadDefaultSuggestions();

    expect(suggestions.map((p) => p.self)).toEqual([
      "/v1/projects/10",
      "/v1/projects/11",
    ]);
  });

  it("degrades to the cached project when the overdue query returns an error", async () => {
    recordLastAssignedProject(project("Projekt Zuletzt", "/v1/projects/99"));
    mockDayliteQueryOverdueProjects.mockResolvedValue({
      status: "error",
      error: {
        code: "SERVER_ERROR",
        httpStatus: 500,
        userMessage: "Serverfehler",
        technicalMessage: "internal server error",
      },
    });

    const suggestions = await loadDefaultSuggestions();

    expect(suggestions.map((p) => p.self)).toEqual(["/v1/projects/99"]);
  });

  it("returns an empty list when the overdue query rejects and no recent project exists", async () => {
    mockDayliteQueryOverdueProjects.mockRejectedValue(
      new Error("network down"),
    );

    expect(await loadDefaultSuggestions()).toEqual([]);
  });
});
