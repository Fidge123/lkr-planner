import { describe, expect, it } from "bun:test";
import type { CalendarCellEvent, EmployeeWeekEvents } from "../generated/tauri";
import {
  applyResolvedProjects,
  createEditSequence,
  groupResults,
  mergeEmployeeRows,
  projectRefsOf,
  relocateEvent,
  removeEvent,
  settlePendingProjects,
  type WeekData,
} from "./week-data";

function event(overrides: Partial<CalendarCellEvent> = {}): CalendarCellEvent {
  return {
    uid: "uid-1",
    kind: "assignment",
    title: "Projekt Nord",
    projectStatus: "in_progress",
    projectCategory: null,
    date: "2026-08-10",
    startTime: "08:00",
    endTime: "12:00",
    href: "/cal/uid-1.ics",
    projectRef: "/v1/projects/1",
    orderIndex: 0,
    projectPending: false,
    ...overrides,
  };
}

function week(eventsByEmployee: Record<string, CalendarCellEvent[]>): WeekData {
  return { eventsByEmployee, errorsByEmployee: {} };
}

function uidsOf(events: CalendarCellEvent[] | undefined): string[] {
  return (events ?? []).map((e) => e.uid);
}

describe("groupResults", () => {
  it("splits each employee into events or an error", () => {
    const entries: EmployeeWeekEvents[] = [
      { employeeReference: "emp-1", events: [event()], error: null },
      { employeeReference: "emp-2", events: [], error: "Kalender fehlt" },
    ];

    expect(groupResults(entries)).toEqual({
      eventsByEmployee: { "emp-1": [event()] },
      errorsByEmployee: { "emp-2": "Kalender fehlt" },
    });
  });
});

describe("projectRefsOf", () => {
  it("lists every distinct project reference of the week once", () => {
    const data = week({
      "emp-1": [
        event({ uid: "a", projectRef: "/v1/projects/1" }),
        event({ uid: "b", projectRef: "/v1/projects/2" }),
      ],
      "emp-2": [
        event({ uid: "c", projectRef: "/v1/projects/1" }),
        event({ uid: "d", kind: "bare", projectRef: null }),
      ],
    });

    expect(projectRefsOf(data).sort()).toEqual([
      "/v1/projects/1",
      "/v1/projects/2",
    ]);
  });
});

describe("applyResolvedProjects", () => {
  it("writes the resolved details into a pending assignment", () => {
    const data = week({
      "emp-1": [
        event({
          title: "Summary",
          projectStatus: null,
          projectPending: true,
        }),
      ],
    });

    const next = applyResolvedProjects(data, {
      "/v1/projects/1": {
        name: "Projekt Nord",
        status: "in_progress",
        category: "Termin FIX geplant",
      },
    });

    expect(next.eventsByEmployee["emp-1"][0]).toMatchObject({
      title: "Projekt Nord",
      projectStatus: "in_progress",
      projectCategory: "Termin FIX geplant",
      projectPending: false,
    });
  });

  it("leaves an unresolvable assignment unresolved but no longer pending", () => {
    const data = week({
      "emp-1": [
        event({ title: "Summary", projectStatus: null, projectPending: true }),
      ],
    });

    const next = applyResolvedProjects(data, { "/v1/projects/1": null });

    expect(next.eventsByEmployee["emp-1"][0]).toMatchObject({
      title: "Summary",
      projectStatus: null,
      projectPending: false,
    });
  });

  it("returns the same week when the details are already in place", () => {
    const data = week({ "emp-1": [event()] });

    const next = applyResolvedProjects(data, {
      "/v1/projects/1": {
        name: "Projekt Nord",
        status: "in_progress",
        category: null,
      },
    });

    expect(next).toBe(data);
  });

  it("leaves assignments of references it was not given untouched", () => {
    const pending = event({ projectStatus: null, projectPending: true });
    const data = week({ "emp-1": [pending] });

    const next = applyResolvedProjects(data, { "/v1/projects/2": null });

    expect(next.eventsByEmployee["emp-1"][0]).toBe(pending);
  });
});

describe("settlePendingProjects", () => {
  it("clears the pending flag of the given references only", () => {
    const data = week({
      "emp-1": [
        event({ uid: "a", projectRef: "/v1/projects/1", projectPending: true }),
        event({ uid: "b", projectRef: "/v1/projects/2", projectPending: true }),
      ],
    });

    const next = settlePendingProjects(data, ["/v1/projects/1"]);

    expect(next.eventsByEmployee["emp-1"].map((e) => e.projectPending)).toEqual(
      [false, true],
    );
  });
});

describe("mergeEmployeeRows", () => {
  it("replaces the refreshed employees and keeps everyone else", () => {
    const data: WeekData = {
      eventsByEmployee: {
        "emp-1": [event({ uid: "old" })],
        "emp-2": [event({ uid: "other" })],
      },
      errorsByEmployee: { "emp-3": "Kalender nicht verfügbar" },
    };

    const next = mergeEmployeeRows(
      data,
      ["emp-1", "emp-3"],
      [
        {
          employeeReference: "emp-1",
          events: [event({ uid: "new" })],
          error: null,
        },
        {
          employeeReference: "emp-3",
          events: [event({ uid: "back" })],
          error: null,
        },
      ],
    );

    expect(uidsOf(next.eventsByEmployee["emp-1"])).toEqual(["new"]);
    expect(uidsOf(next.eventsByEmployee["emp-2"])).toEqual(["other"]);
    expect(uidsOf(next.eventsByEmployee["emp-3"])).toEqual(["back"]);
    expect(next.errorsByEmployee).toEqual({});
  });

  it("drops a refreshed employee that the backend no longer returns", () => {
    const data = week({ "emp-1": [event()] });

    const next = mergeEmployeeRows(data, ["emp-1"], []);

    expect(next.eventsByEmployee["emp-1"]).toBeUndefined();
  });

  it("shows the error of a refreshed employee instead of their old events", () => {
    const data = week({ "emp-1": [event()] });

    const next = mergeEmployeeRows(
      data,
      ["emp-1"],
      [{ employeeReference: "emp-1", events: [], error: "Kalender fehlt" }],
    );

    expect(next.eventsByEmployee["emp-1"]).toBeUndefined();
    expect(next.errorsByEmployee["emp-1"]).toBe("Kalender fehlt");
  });
});

describe("removeEvent", () => {
  it("removes the event from whichever employee holds it", () => {
    const data = week({
      "emp-1": [event({ uid: "a" }), event({ uid: "b" })],
      "emp-2": [event({ uid: "c" })],
    });

    const next = removeEvent(data, "b");

    expect(uidsOf(next.eventsByEmployee["emp-1"])).toEqual(["a"]);
    expect(uidsOf(next.eventsByEmployee["emp-2"])).toEqual(["c"]);
  });
});

describe("relocateEvent", () => {
  const monday = "2026-08-10";
  const tuesday = "2026-08-11";

  function cell(data: WeekData, employee: string, date: string) {
    return (data.eventsByEmployee[employee] ?? [])
      .filter((e) => e.date === date && e.kind === "assignment")
      .sort((a, b) => (a.orderIndex ?? 0) - (b.orderIndex ?? 0));
  }

  it("inserts the event at the target position and renumbers the cell", () => {
    const moved = event({ uid: "moved", date: monday, orderIndex: 0 });
    const data = week({
      "emp-1": [moved],
      "emp-2": [
        event({ uid: "first", date: tuesday, orderIndex: 0 }),
        event({ uid: "second", date: tuesday, orderIndex: 1 }),
      ],
    });

    const next = relocateEvent(data, moved, {
      employeeRef: "emp-2",
      date: tuesday,
      orderIndex: 1,
    });

    expect(uidsOf(next.eventsByEmployee["emp-1"])).toEqual([]);
    const target = cell(next, "emp-2", tuesday);
    expect(uidsOf(target)).toEqual(["first", "moved", "second"]);
    expect(target.map((e) => e.orderIndex)).toEqual([0, 1, 2]);
  });

  it("clears the times of the moved event until the day is re-slotted", () => {
    const moved = event({ uid: "moved", date: monday });
    const data = week({ "emp-1": [moved] });

    const next = relocateEvent(data, moved, {
      employeeRef: "emp-1",
      date: tuesday,
      orderIndex: 0,
    });

    expect(next.eventsByEmployee["emp-1"]).toEqual([
      {
        ...moved,
        date: tuesday,
        startTime: null,
        endTime: null,
        orderIndex: 0,
      },
    ]);
  });

  it("reorders within the same cell", () => {
    const a = event({ uid: "a", orderIndex: 0 });
    const b = event({ uid: "b", orderIndex: 1 });
    const c = event({ uid: "c", orderIndex: 2 });
    const data = week({ "emp-1": [a, b, c] });

    const next = relocateEvent(data, a, {
      employeeRef: "emp-1",
      date: monday,
      orderIndex: 2,
    });

    expect(uidsOf(cell(next, "emp-1", monday))).toEqual(["b", "c", "a"]);
  });

  it("keeps absences and bare events of the target cell in place", () => {
    const moved = event({ uid: "moved", date: monday });
    const absence = event({
      uid: "abs",
      kind: "absence",
      date: tuesday,
      projectRef: null,
      orderIndex: null,
    });
    const data = week({ "emp-1": [moved, absence] });

    const next = relocateEvent(data, moved, {
      employeeRef: "emp-1",
      date: tuesday,
      orderIndex: 0,
    });

    expect(next.eventsByEmployee["emp-1"]).toContainEqual(absence);
  });

  it("leaves a fixed appointment in place when the drop changes its day", () => {
    const fixed = event({
      uid: "fixed",
      date: monday,
      projectCategory: "Termin FIX geplant",
    });
    const data = week({ "emp-1": [fixed], "emp-2": [] });

    const next = relocateEvent(data, fixed, {
      employeeRef: "emp-2",
      date: tuesday,
      orderIndex: 0,
    });

    expect(next).toBe(data);
  });

  it("moves a fixed appointment to another employee on its own day", () => {
    const fixed = event({
      uid: "fixed",
      date: monday,
      projectCategory: "Termin FIX geplant",
    });
    const data = week({ "emp-1": [fixed], "emp-2": [] });

    const next = relocateEvent(data, fixed, {
      employeeRef: "emp-2",
      date: monday,
      orderIndex: 0,
    });

    expect(uidsOf(next.eventsByEmployee["emp-1"])).toEqual([]);
    expect(uidsOf(next.eventsByEmployee["emp-2"])).toEqual(["fixed"]);
  });

  it("inserts an event that came from another week", () => {
    const fromOtherWeek = event({ uid: "moved", date: "2026-08-03" });
    const data = week({ "emp-1": [] });

    const next = relocateEvent(data, fromOtherWeek, {
      employeeRef: "emp-1",
      date: monday,
      orderIndex: 0,
    });

    expect(uidsOf(next.eventsByEmployee["emp-1"])).toEqual(["moved"]);
  });
});

describe("createEditSequence", () => {
  it("keeps a refresh requested after the last edit current", () => {
    const sequence = createEditSequence();
    sequence.markEdit();

    const isCurrent = sequence.requestRefresh();

    expect(isCurrent()).toBe(true);
  });

  it("supersedes a refresh once another edit follows it", () => {
    const sequence = createEditSequence();
    const isCurrent = sequence.requestRefresh();

    sequence.markEdit();

    expect(isCurrent()).toBe(false);
  });
});
