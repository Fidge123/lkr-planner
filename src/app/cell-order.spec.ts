import { describe, expect, it } from "bun:test";
import { assignmentPositions, sortCellEvents } from "./cell-order";
import type { CellEvent } from "./types";

function assignment(
  uid: string,
  orderIndex: number | null,
  times: [string, string] | null = null,
): CellEvent {
  return {
    uid,
    kind: "assignment",
    title: `Projekt ${uid}`,
    color: "bg-primary",
    startTime: times?.[0] ?? null,
    endTime: times?.[1] ?? null,
    href: `/calendars/emp-a/${uid}.ics`,
    projectRef: "/v1/projects/1",
    projectStatus: "in_progress",
    categoryColor: null,
    orderIndex,
  };
}

function absence(uid: string): CellEvent {
  return {
    uid,
    kind: "absence",
    title: "Urlaub",
    color: "bg-info/30",
    startTime: null,
    endTime: null,
    href: null,
    projectRef: null,
    projectStatus: null,
    categoryColor: null,
    orderIndex: null,
  };
}

describe("sortCellEvents", () => {
  it("renders a cell's assignments in order-index order", () => {
    const sorted = sortCellEvents([
      assignment("uid-c", 2),
      assignment("uid-a", 0),
      assignment("uid-b", 1),
    ]);

    expect(sorted.map((event) => event.uid)).toEqual([
      "uid-a",
      "uid-b",
      "uid-c",
    ]);
  });

  it("sorts assignments without an order index last, by uid", () => {
    const sorted = sortCellEvents([
      assignment("uid-z", null),
      assignment("uid-b", 5),
      assignment("uid-a", null),
    ]);

    expect(sorted.map((event) => event.uid)).toEqual([
      "uid-b",
      "uid-a",
      "uid-z",
    ]);
  });

  it("leaves absences in the positions the backend gave them", () => {
    const sorted = sortCellEvents([
      absence("abs-1"),
      assignment("uid-b", 1),
      assignment("uid-a", 0),
    ]);

    expect(sorted.map((event) => event.uid)).toEqual([
      "abs-1",
      "uid-a",
      "uid-b",
    ]);
  });

  it("places an assignment excluded from re-slotting by its index, stale times and all", () => {
    // The excluded card keeps whatever times it had, so they can disagree with its
    // position; the order index alone decides where it renders.
    const sorted = sortCellEvents([
      assignment("uid-late", 1, ["08:00", "16:00"]),
      assignment("uid-early", 0, ["08:00", "12:00"]),
    ]);

    expect(sorted.map((event) => event.uid)).toEqual(["uid-early", "uid-late"]);
    expect(sorted[1].startTime).toBe("08:00");
  });

  it("does not mutate the input", () => {
    const events = [assignment("uid-b", 1), assignment("uid-a", 0)];

    sortCellEvents(events);

    expect(events.map((event) => event.uid)).toEqual(["uid-b", "uid-a"]);
  });
});

describe("assignmentPositions", () => {
  it("numbers assignments consecutively, skipping other event kinds", () => {
    const positions = assignmentPositions([
      absence("abs-1"),
      assignment("uid-a", 0),
      assignment("uid-b", 1),
    ]);

    expect(positions.get("uid-a")).toBe(0);
    expect(positions.get("uid-b")).toBe(1);
    expect(positions.has("abs-1")).toBe(false);
  });
});
