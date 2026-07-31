import { describe, expect, it } from "bun:test";
import type { CalendarCellEvent } from "../generated/tauri";
import {
  assignmentBackgroundColor,
  type CellEvent,
  hasAbsenceConflict,
  oklchFromHex,
  toCellEvent,
} from "./types";

function calendarEvent(
  overrides: Partial<CalendarCellEvent> = {},
): CalendarCellEvent {
  return {
    uid: "uid-1",
    kind: "absence",
    title: "UB",
    projectStatus: null,
    categoryColor: null,
    date: "2026-05-05",
    startTime: null,
    endTime: null,
    href: null,
    projectRef: null,
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
  it("carries the Daylite category color instead of a status class", () => {
    const event = toCellEvent(
      calendarEvent({
        kind: "assignment",
        title: "Projekt Nord",
        projectStatus: "in_progress",
        categoryColor: "#8bc34a",
      }),
    );

    expect(event.categoryColor).toBe("#8bc34a");
    expect(event.color).toBe("");
  });

  it("carries no color class without a category color either", () => {
    const event = toCellEvent(
      calendarEvent({
        kind: "assignment",
        title: "Projekt Nord",
        projectStatus: "in_progress",
      }),
    );

    expect(event.categoryColor).toBeNull();
    expect(event.color).toBe("");
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
      expect(event.color).toBe("");
    }
  });

  it("carries no color class for an unresolved project", () => {
    const event = toCellEvent(
      calendarEvent({ kind: "assignment", title: "Projekt Nord" }),
    );

    expect(event.categoryColor).toBeNull();
    expect(event.color).toBe("");
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

describe("assignmentBackgroundColor", () => {
  it("returns a plain hex color, never a CSS color function", () => {
    const color = assignmentBackgroundColor("#8bc34a");
    expect(color).toMatch(/^#[0-9a-f]{6}$/);
    expect(color).not.toContain("var(");
    expect(color).not.toContain("oklch");
  });

  it("keeps the hue of the Daylite color", () => {
    expect(oklchFromHex(assignmentBackgroundColor("#e8352a"))?.hue).toBeCloseTo(
      29,
      -0.5,
    );
    expect(oklchFromHex(assignmentBackgroundColor("#3b7dd8"))?.hue).toBeCloseTo(
      257,
      -0.5,
    );
  });

  it("renders every category at the same lightness", () => {
    const lightnesses = ["#2211aa", "#e8352a", "#7ec8f0", "#ffffff"].map(
      (hex) => oklchFromHex(assignmentBackgroundColor(hex))?.lightness ?? 0,
    );

    for (const lightness of lightnesses) {
      expect(lightness).toBeCloseTo(0.58, 1);
    }
  });

  it("caps the chroma of a harsh color", () => {
    const { chroma } = oklchFromHex(assignmentBackgroundColor("#e8352a")) ?? {
      chroma: 1,
    };
    expect(chroma).toBeLessThanOrEqual(0.141);
  });

  it("leaves a muted color's chroma below the cap", () => {
    const { chroma } = oklchFromHex(assignmentBackgroundColor("#7ec8f0")) ?? {
      chroma: 1,
    };
    expect(chroma).toBeLessThan(0.14);
    expect(chroma).toBeGreaterThan(0);
  });

  it("gives an assignment without a category an achromatic color", () => {
    const color = assignmentBackgroundColor(null);
    const [, r, g, b] = color.match(/^#(..)(..)(..)$/) ?? [];
    expect(r).toBe(g);
    expect(g).toBe(b);
  });

  it("renders the no-category color at the same lightness as a category", () => {
    const neutral = oklchFromHex(assignmentBackgroundColor(null))?.lightness;
    expect(neutral).toBeCloseTo(0.58, 1);
  });

  it("keeps the no-category color clearly darker than a bare event surface", () => {
    // Bare events use bg-base-200, oklch(93%) in the light theme.
    const neutral =
      oklchFromHex(assignmentBackgroundColor(null))?.lightness ?? 1;
    expect(0.93 - neutral).toBeGreaterThan(0.25);
  });

  it("falls back to the achromatic color for an unparsable value", () => {
    expect(assignmentBackgroundColor("nicht-eine-farbe")).toBe(
      assignmentBackgroundColor(null),
    );
  });

  it("keeps different hues distinguishable", () => {
    expect(assignmentBackgroundColor("#e8352a")).not.toBe(
      assignmentBackgroundColor("#3b7dd8"),
    );
  });

  it("stays inside the sRGB gamut", () => {
    for (const hex of ["#00ff00", "#ffff00", "#0000ff", "#ff00ff"]) {
      expect(assignmentBackgroundColor(hex)).toMatch(/^#[0-9a-f]{6}$/);
    }
  });
});
