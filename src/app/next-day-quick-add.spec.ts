import { describe, expect, it } from "bun:test";
import type { CalendarCellEvent } from "../generated/tauri";
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

function event(
  date: string,
  kind: CalendarCellEvent["kind"] = "assignment",
): CalendarCellEvent {
  return {
    uid: `uid-${date}`,
    kind,
    title: "Irrelevant",
    projectStatus: null,
    projectRef: null,
    date,
    startTime: null,
    endTime: null,
    href: null,
  };
}

// ── 2.1 – target day resolution and the last-visible-day boundary ─────────────
describe("nextVisibleDay", () => {
  it("returns the following day within the visible week", () => {
    expect(nextVisibleDay(weekDays, "2026-05-05")).toBe("2026-05-06");
  });

  it("returns null when the date is the last visible day", () => {
    expect(nextVisibleDay(weekDays, "2026-05-08")).toBeNull();
  });

  it("returns null when the date is not part of the visible week", () => {
    expect(nextVisibleDay(weekDays, "2026-05-11")).toBeNull();
  });
});

// ── 1.1 / 1.3 / 2.1 / 4.3 / 4.4 / 5.1 – ghost lifecycle ────────────────────────
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

  it("stays null after an edit when no ghost exists", () => {
    expect(nextGhostState(null, { kind: "edit" }, weekDays)).toBeNull();
  });

  it("clears the ghost after a delete", () => {
    const existing: GhostSuggestion = {
      date: "2026-05-06",
      projectRef: "/v1/projects/1",
      projectName: "Projekt Nord",
    };

    expect(nextGhostState(existing, { kind: "delete" }, weekDays)).toBeNull();
  });

  it("chains: clicking a ghost re-creates on its own date and advances the ghost one day further", () => {
    const clicked: GhostSuggestion = {
      date: "2026-05-06",
      projectRef: "/v1/projects/1",
      projectName: "Projekt Nord",
    };

    const chained = nextGhostState(
      clicked,
      {
        kind: "create",
        date: clicked.date,
        projectRef: clicked.projectRef,
        projectName: clicked.projectName,
      },
      weekDays,
    );

    expect(chained).toEqual({
      date: "2026-05-07",
      projectRef: "/v1/projects/1",
      projectName: "Projekt Nord",
    });
  });

  it("ends the chain when the clicked ghost sits on the last visible day", () => {
    const clicked: GhostSuggestion = {
      date: "2026-05-08",
      projectRef: "/v1/projects/1",
      projectName: "Projekt Nord",
    };

    const chained = nextGhostState(
      clicked,
      {
        kind: "create",
        date: clicked.date,
        projectRef: clicked.projectRef,
        projectName: clicked.projectName,
      },
      weekDays,
    );

    expect(chained).toBeNull();
  });
});

// ── 2.3 – suppression when the target day already holds an event ──────────────
describe("isGhostVisible", () => {
  const ghost: GhostSuggestion = {
    date: "2026-05-06",
    projectRef: "/v1/projects/1",
    projectName: "Projekt Nord",
  };

  it("is visible on its target day when that day holds no events", () => {
    expect(isGhostVisible(ghost, "2026-05-06", [])).toBe(true);
  });

  it("is not visible on a different day", () => {
    expect(isGhostVisible(ghost, "2026-05-07", [])).toBe(false);
  });

  it("is suppressed when the target day already holds an assignment", () => {
    expect(isGhostVisible(ghost, "2026-05-06", [event("2026-05-06")])).toBe(
      false,
    );
  });

  it("is suppressed when the target day already holds an absence", () => {
    expect(
      isGhostVisible(ghost, "2026-05-06", [event("2026-05-06", "absence")]),
    ).toBe(false);
  });

  it("is never visible when there is no ghost", () => {
    expect(isGhostVisible(null, "2026-05-06", [])).toBe(false);
  });
});
