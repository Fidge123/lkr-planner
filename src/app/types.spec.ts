import { describe, expect, it } from "bun:test";
import type { CalendarCellEvent } from "../generated/tauri";
import { type CellEvent, hasAbsenceConflict, toCellEvent } from "./types";

function calendarEvent(
  overrides: Partial<CalendarCellEvent> = {},
): CalendarCellEvent {
  return {
    uid: "uid-1",
    kind: "absence",
    title: "UB",
    projectStatus: null,
    categoryColor: null,
    projectCategory: null,
    date: "2026-05-05",
    startTime: null,
    endTime: null,
    href: null,
    projectRef: null,
    orderIndex: null,
    ...overrides,
  };
}

function cellEvent(overrides: Partial<CellEvent> = {}): CellEvent {
  return {
    uid: "uid-1",
    kind: "assignment",
    title: "Projekt Nord",
    color: "bg-primary",
    startTime: null,
    endTime: null,
    href: null,
    projectRef: null,
    categoryColor: null,
    projectStatus: null,
    orderIndex: null,
    ...overrides,
  };
}

describe("hasAbsenceConflict", () => {
  it("flags an absence together with an assignment", () => {
    const events = [
      cellEvent({ uid: "abs", kind: "absence", title: "UB" }),
      cellEvent({ uid: "asg", kind: "assignment" }),
    ];

    expect(hasAbsenceConflict(events)).toBe(true);
  });

  it("flags an absence together with a bare event", () => {
    const events = [
      cellEvent({ uid: "abs", kind: "absence", title: "KR" }),
      cellEvent({ uid: "bare", kind: "bare", title: "Werkstatt" }),
    ];

    expect(hasAbsenceConflict(events)).toBe(true);
  });

  it("does not flag an absence on its own", () => {
    const events = [cellEvent({ uid: "abs", kind: "absence", title: "UB" })];

    expect(hasAbsenceConflict(events)).toBe(false);
  });

  it("does not flag several absences without an appointment", () => {
    const events = [
      cellEvent({ uid: "abs-1", kind: "absence", title: "UB" }),
      cellEvent({ uid: "abs-2", kind: "absence", title: "KR" }),
    ];

    expect(hasAbsenceConflict(events)).toBe(false);
  });

  it("does not flag appointments without an absence", () => {
    const events = [
      cellEvent({ uid: "asg", kind: "assignment" }),
      cellEvent({ uid: "bare", kind: "bare" }),
    ];

    expect(hasAbsenceConflict(events)).toBe(false);
  });

  it("does not flag an empty cell", () => {
    expect(hasAbsenceConflict([])).toBe(false);
  });
});

describe("toCellEvent absence colors", () => {
  const codeColors: [string, string][] = [
    ["UB", "bg-(--color-absence-vacation)/50"],
    ["SU", "bg-(--color-absence-vacation)/30"],
    ["UU", "bg-(--color-absence-vacation)/15"],
    ["KR", "bg-(--color-absence-sick)/40"],
    ["Kro", "bg-(--color-absence-sick)/20"],
    ["FA", "bg-(--color-absence-special)/30"],
  ];

  for (const [title, expected] of codeColors) {
    it(`maps absence code ${title} to its family color`, () => {
      expect(toCellEvent(calendarEvent({ title })).color).toBe(expected);
    });
  }

  it("matches codes case-insensitively and ignores surrounding whitespace", () => {
    expect(toCellEvent(calendarEvent({ title: "  kro  " })).color).toBe(
      "bg-(--color-absence-sick)/20",
    );
  });

  it("keeps the default color for an unknown absence title", () => {
    expect(toCellEvent(calendarEvent({ title: "Sonderfall" })).color).toBe(
      "bg-info/30",
    );
  });

  it("does not apply absence colors to bare events", () => {
    expect(
      toCellEvent(calendarEvent({ kind: "bare", title: "UB" })).color,
    ).toBe("bg-base-200");
  });
});

describe("toCellEvent assignment colors", () => {
  it("carries the Daylite category color untouched", () => {
    const event = toCellEvent(
      calendarEvent({
        kind: "assignment",
        title: "Projekt Nord",
        projectStatus: "in_progress",
        categoryColor: "#8bc34a",
      }),
    );

    expect(event.categoryColor).toBe("#8bc34a");
    expect(event.color).toBe("bg-base-200");
  });

  it("uses the neutral surface without a category color", () => {
    const event = toCellEvent(
      calendarEvent({
        kind: "assignment",
        title: "Projekt Nord",
        projectStatus: "in_progress",
      }),
    );

    expect(event.categoryColor).toBeNull();
    expect(event.color).toBe("bg-base-200");
  });

  it("ignores the project status entirely", () => {
    const statuses = [
      "new_status",
      "in_progress",
      "done",
      "abandoned",
      "cancelled",
      "deferred",
    ];

    for (const projectStatus of statuses) {
      const event = toCellEvent(
        calendarEvent({ kind: "assignment", title: "Projekt", projectStatus }),
      );
      expect(event.color).toBe("bg-base-200");
    }
  });

  it("uses the neutral surface for an unresolved project", () => {
    const event = toCellEvent(
      calendarEvent({ kind: "assignment", title: "Projekt Nord" }),
    );

    expect(event.categoryColor).toBeNull();
    expect(event.color).toBe("bg-base-200");
  });

  it("ignores a category color on absence and bare events", () => {
    const absence = toCellEvent(calendarEvent({ categoryColor: "#8bc34a" }));
    const bare = toCellEvent(
      calendarEvent({
        kind: "bare",
        title: "Werkstatt",
        categoryColor: "#8bc34a",
      }),
    );

    expect(absence.categoryColor).toBeNull();
    expect(bare.categoryColor).toBeNull();
  });
});
