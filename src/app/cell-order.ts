import type { CellEvent } from "./types";

/**
 * Orders a cell's assignment cards the way the backend's `ordering_key` orders the day, so
 * the visual order and the allocated time slots agree. Absences and bare events keep the
 * positions the backend gave them.
 */
export function sortCellEvents(events: CellEvent[]): CellEvent[] {
  const assignments = events
    .filter((event) => event.kind === "assignment")
    .sort(byCellOrder);
  let next = 0;
  return events.map((event) =>
    event.kind === "assignment" ? assignments[next++] : event,
  );
}

/** Position of each assignment among the cell's assignments, keyed by UID. */
export function assignmentPositions(events: CellEvent[]): Map<string, number> {
  const positions = new Map<string, number>();
  for (const event of events) {
    if (event.kind === "assignment") {
      positions.set(event.uid, positions.size);
    }
  }
  return positions;
}

/**
 * Assignments are ordered by their persisted index. Two that share one, which happens while
 * an assignment excluded from re-slotting keeps a stale index, are read the way a planner
 * reads the cell: the earlier start comes first, a longer assignment wins an identical
 * start, and the UID keeps the ordering total. Unindexed and untimed assignments sort last.
 */
function byCellOrder(a: CellEvent, b: CellEvent): number {
  return (
    orderIndexOf(a) - orderIndexOf(b) ||
    startMinutesOf(a) - startMinutesOf(b) ||
    durationMinutesOf(b) - durationMinutesOf(a) ||
    // Code-unit order, matching the backend's bytewise comparison of the UID.
    (a.uid < b.uid ? -1 : a.uid > b.uid ? 1 : 0)
  );
}

function orderIndexOf(event: CellEvent): number {
  return event.orderIndex ?? Number.MAX_SAFE_INTEGER;
}

function startMinutesOf(event: CellEvent): number {
  return minutesOfDay(event.startTime) ?? Number.MAX_SAFE_INTEGER;
}

function durationMinutesOf(event: CellEvent): number {
  const start = minutesOfDay(event.startTime);
  const end = minutesOfDay(event.endTime);
  if (start === null || end === null) return 0;
  return Math.max(end - start, 0);
}

/** Minutes since midnight for an `HH:MM` time, null for an all-day or malformed one. */
function minutesOfDay(time: string | null): number | null {
  if (time === null) return null;
  const [hours, minutes] = time.split(":");
  const total = Number(hours) * 60 + Number(minutes);
  return Number.isFinite(total) ? total : null;
}
