import { describe, expect, it } from "bun:test";
import {
  type GhostSuggestion,
  isGhostVisible,
  nextGhostState,
  nextVisibleDay,
} from "./next-day-quick-add";

const weekDays = [
  "2026-05-04",
  "2026-05-05",
  "2026-05-06",
  "2026-05-07",
  "2026-05-08",
];

describe("nextVisibleDay", () => {
  it("returns null when the date is not part of the visible week", () => {
    expect(nextVisibleDay(weekDays, "2026-05-11")).toBeNull();
  });
});

describe("nextGhostState", () => {
  it("sets a ghost on the next visible day after a create", () => {
    const result = nextGhostState(
      null,
      {
        kind: "create",
        date: "2026-05-05",
        projectRef: "/v1/projects/1",
        projectName: "Projekt Nord",
      },
      weekDays,
    );

    expect(result).toEqual({
      date: "2026-05-06",
      projectRef: "/v1/projects/1",
      projectName: "Projekt Nord",
    });
  });

  it("sets no ghost when creating on the last visible day", () => {
    const result = nextGhostState(
      null,
      {
        kind: "create",
        date: "2026-05-08",
        projectRef: "/v1/projects/1",
        projectName: "Projekt Nord",
      },
      weekDays,
    );

    expect(result).toBeNull();
  });

  it("leaves an existing ghost untouched after an edit", () => {
    const existing: GhostSuggestion = {
      date: "2026-05-06",
      projectRef: "/v1/projects/1",
      projectName: "Projekt Nord",
    };

    expect(nextGhostState(existing, { kind: "edit" }, weekDays)).toBe(existing);
  });

  it("clears the ghost after a delete", () => {
    const existing: GhostSuggestion = {
      date: "2026-05-06",
      projectRef: "/v1/projects/1",
      projectName: "Projekt Nord",
    };

    expect(nextGhostState(existing, { kind: "delete" }, weekDays)).toBeNull();
  });
});

describe("isGhostVisible", () => {
  const ghost: GhostSuggestion = {
    date: "2026-05-06",
    projectRef: "/v1/projects/1",
    projectName: "Projekt Nord",
  };

  it("is visible on its target day when that day holds no events", () => {
    expect(isGhostVisible(ghost, "2026-05-06", 0)).toBe(true);
  });

  it("is not visible on a different day", () => {
    expect(isGhostVisible(ghost, "2026-05-07", 0)).toBe(false);
  });

  it("is suppressed when the target day already holds an assignment or absence", () => {
    expect(isGhostVisible(ghost, "2026-05-06", 1)).toBe(false);
  });

  it("is never visible when there is no ghost", () => {
    expect(isGhostVisible(null, "2026-05-06", 0)).toBe(false);
  });
});
