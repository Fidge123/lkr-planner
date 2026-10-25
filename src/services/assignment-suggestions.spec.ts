import { describe, expect, it } from "bun:test";
import type { DayliteProjectSummary } from "../generated/tauri";
import { combineSuggestions } from "./assignment-suggestions";

function project(name: string, ref: string): DayliteProjectSummary {
  return { self: ref, name };
}

describe("combineSuggestions", () => {
  const overdue = [
    project("Projekt 10", "/v1/projects/10"),
    project("Projekt 11", "/v1/projects/11"),
    project("Projekt 12", "/v1/projects/12"),
    project("Projekt 13", "/v1/projects/13"),
    project("Projekt 14", "/v1/projects/14"),
  ];

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
