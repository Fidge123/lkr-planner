import { describe, expect, it } from "bun:test";
import type { CalendarCellEvent } from "../generated/tauri";
import {
  activeHighlightForWeek,
  type CellEvent,
  hasAbsenceConflict,
  highlightKey,
  isHighlighted,
  nextHighlight,
  toCellEvent,
} from "./types";

const categoryColors = { Bau: "#8bc34a" };

function calendarEvent(
  overrides: Partial<CalendarCellEvent> = {},
): CalendarCellEvent {
  return {
    uid: "uid-1",
    kind: "absence",
    title: "UB",
    projectStatus: null,
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
      expect(toCellEvent(calendarEvent({ title }), categoryColors).color).toBe(
        expected,
      );
    });
  }

  it("matches codes case-insensitively and ignores surrounding whitespace", () => {
    expect(
      toCellEvent(calendarEvent({ title: "  kro  " }), categoryColors).color,
    ).toBe("bg-(--color-absence-sick)/20");
  });

  it("keeps the default color for an unknown absence title", () => {
    expect(
      toCellEvent(calendarEvent({ title: "Sonderfall" }), categoryColors).color,
    ).toBe("bg-info/30");
  });

  it("does not apply absence colors to bare events", () => {
    expect(
      toCellEvent(calendarEvent({ kind: "bare", title: "UB" }), categoryColors)
        .color,
    ).toBe("bg-base-200");
  });
});

describe("toCellEvent assignment colors", () => {
  it("colors the card from the category's Daylite color", () => {
    const event = toCellEvent(
      calendarEvent({
        kind: "assignment",
        title: "Projekt Nord",
        projectStatus: "in_progress",
        projectCategory: "Bau",
      }),
      categoryColors,
    );

    expect(event.categoryColor).toBe("#8bc34a");
    expect(event.color).toBe("bg-base-200");
  });

  it("leaves the strip unset for a category that has no color", () => {
    const event = toCellEvent(
      calendarEvent({
        kind: "assignment",
        title: "Projekt Nord",
        projectStatus: "in_progress",
        projectCategory: "Ohne Farbe",
      }),
      categoryColors,
    );

    expect(event.categoryColor).toBeNull();
  });

  it("uses the neutral surface without a category", () => {
    const event = toCellEvent(
      calendarEvent({
        kind: "assignment",
        title: "Projekt Nord",
        projectStatus: "in_progress",
      }),
      categoryColors,
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
        categoryColors,
      );
      expect(event.color).toBe("bg-base-200");
    }
  });

  it("uses the neutral surface for an unresolved project", () => {
    const event = toCellEvent(
      calendarEvent({ kind: "assignment", title: "Projekt Nord" }),
      categoryColors,
    );

    expect(event.categoryColor).toBeNull();
    expect(event.color).toBe("bg-base-200");
  });

  it("ignores a category on absence and bare events", () => {
    const absence = toCellEvent(
      calendarEvent({ projectCategory: "Bau" }),
      categoryColors,
    );
    const bare = toCellEvent(
      calendarEvent({
        kind: "bare",
        title: "Werkstatt",
        projectCategory: "Bau",
      }),
      categoryColors,
    );

    expect(absence.categoryColor).toBeNull();
    expect(bare.categoryColor).toBeNull();
  });
});

describe("highlightKey", () => {
  it("keys an assignment by its Daylite project reference", () => {
    const event = cellEvent({ projectRef: "/v1/projects/42" });

    expect(highlightKey(event)).toBe("/v1/projects/42");
  });

  it("keys an unresolved assignment by its reference as well", () => {
    const event = cellEvent({
      projectRef: "/v1/projects/42",
      projectStatus: null,
    });

    expect(highlightKey(event)).toBe("/v1/projects/42");
  });

  it("gives a bare event no key", () => {
    expect(highlightKey(cellEvent({ kind: "bare", title: "Werkstatt" }))).toBe(
      null,
    );
  });

  it("gives an absence no key", () => {
    expect(highlightKey(cellEvent({ kind: "absence", title: "UB" }))).toBe(
      null,
    );
  });
});

describe("isHighlighted", () => {
  it("highlights an assignment holding the active reference", () => {
    const event = cellEvent({ projectRef: "/v1/projects/42" });

    expect(isHighlighted(event, "/v1/projects/42")).toBe(true);
  });

  it("leaves an assignment of another project alone", () => {
    const event = cellEvent({ projectRef: "/v1/projects/7" });

    expect(isHighlighted(event, "/v1/projects/42")).toBe(false);
  });

  it("highlights nothing while no highlight is active", () => {
    const event = cellEvent({ projectRef: "/v1/projects/42" });

    expect(isHighlighted(event, null)).toBe(false);
  });

  it("never highlights events without a reference, even against each other", () => {
    const bare = cellEvent({ kind: "bare", title: "Werkstatt" });
    const absence = cellEvent({ kind: "absence", title: "UB" });

    expect(isHighlighted(bare, null)).toBe(false);
    expect(isHighlighted(absence, null)).toBe(false);
    expect(isHighlighted(bare, highlightKey(absence))).toBe(false);
    expect(isHighlighted(absence, highlightKey(bare))).toBe(false);
  });
});

describe("nextHighlight", () => {
  it("activates a highlight from nothing", () => {
    expect(nextHighlight(null, "/v1/projects/42")).toBe("/v1/projects/42");
  });

  it("replaces an active highlight with another project", () => {
    expect(nextHighlight("/v1/projects/7", "/v1/projects/42")).toBe(
      "/v1/projects/42",
    );
  });

  it("clears the highlight when the active project is picked again", () => {
    expect(nextHighlight("/v1/projects/42", "/v1/projects/42")).toBe(null);
  });

  it("leaves the highlight untouched for an event without a reference", () => {
    expect(nextHighlight("/v1/projects/42", null)).toBe("/v1/projects/42");
    expect(nextHighlight(null, null)).toBe(null);
  });
});

describe("activeHighlightForWeek", () => {
  const highlight = { key: "/v1/projects/42", weekStart: "2026-09-07" };

  it("applies a highlight in the week it was activated in", () => {
    expect(activeHighlightForWeek(highlight, "2026-09-07")).toBe(
      "/v1/projects/42",
    );
  });

  it("drops it in any other week", () => {
    expect(activeHighlightForWeek(highlight, "2026-09-14")).toBe(null);
    expect(activeHighlightForWeek(highlight, "2026-08-31")).toBe(null);
  });

  it("applies nothing when no highlight was activated", () => {
    expect(activeHighlightForWeek(null, "2026-09-07")).toBe(null);
  });
});
