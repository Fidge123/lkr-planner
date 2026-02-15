import { describe, expect, it } from "bun:test";
import type { DayliteProjectRecord } from "../domain/planning";
import {
  applyStandardFilter,
  defaultStandardFilter,
  matchesStandardFilter,
} from "./standard-filter";

describe("standard filter rule engine", () => {
  it("uses documented default rules", () => {
    expect(defaultStandardFilter).toEqual({
      pipelines: ["Aufträge"],
      columns: ["Vorbereitung", "Durchführung"],
      categories: ["Überfällig", "Liefertermin bekannt"],
      exclusionStatuses: ["Done"],
    });
  });

  it("includes a project when pipeline/column rule matches", () => {
    const project = createProject({
      keywords: ["Aufträge", "Vorbereitung"],
      category: "Beliebig",
      status: "in_progress",
    });

    expect(matchesStandardFilter(project)).toBe(true);
  });

  it("includes a project when category rule matches", () => {
    const project = createProject({
      keywords: ["Irgendwas"],
      category: "Überfällig",
      status: "new_status",
    });

    expect(matchesStandardFilter(project)).toBe(true);
  });

  it("excludes done projects even if another rule matches", () => {
    const project = createProject({
      keywords: ["Aufträge", "Vorbereitung"],
      category: "Überfällig",
      status: "done",
    });

    expect(matchesStandardFilter(project)).toBe(false);
  });

  it("filters non-matching projects from project lists", () => {
    const projects: DayliteProjectRecord[] = [
      createProject({
        self: "/v1/projects/7101",
        keywords: ["Aufträge", "Durchführung"],
        category: "Neutral",
        status: "in_progress",
      }),
      createProject({
        self: "/v1/projects/7102",
        keywords: ["Sonstiges"],
        category: "Liefertermin bekannt",
        status: "new_status",
      }),
      createProject({
        self: "/v1/projects/7103",
        keywords: ["Aufträge", "Vorbereitung"],
        category: "Überfällig",
        status: "done",
      }),
      createProject({
        self: "/v1/projects/7104",
        keywords: ["Sonstiges"],
        category: "Neutral",
        status: "in_progress",
      }),
    ];

    const filtered = applyStandardFilter(projects);

    expect(filtered.map((project) => project.self)).toEqual([
      "/v1/projects/7101",
      "/v1/projects/7102",
    ]);
  });
});

function createProject(
  overrides: Partial<DayliteProjectRecord> = {},
): DayliteProjectRecord {
  return {
    self: overrides.self ?? "/v1/projects/7000",
    name: overrides.name ?? "Projekt",
    status: overrides.status ?? "new_status",
    category: overrides.category,
    keywords: overrides.keywords,
  };
}
