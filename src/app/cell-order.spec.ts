import { describe, expect, it } from "bun:test";
import { assignmentPositions } from "./cell-order";
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
    projectPending: false,
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
    projectPending: false,
  };
}

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
