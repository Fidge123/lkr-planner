import type { CellEvent } from "./types";

/**
 * Orders a cell's assignment cards by their persisted order index, mirroring the backend's
 * (index, UID) rule so the visual order and the allocated time slots agree. Absences and bare
 * events keep the positions the backend gave them.
 */
export function sortCellEvents(events: CellEvent[]): CellEvent[] {
  const assignments = events
    .filter((event) => event.kind === "assignment")
    .sort(byOrderIndex);
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

function byOrderIndex(a: CellEvent, b: CellEvent): number {
  const left = a.orderIndex ?? Number.MAX_SAFE_INTEGER;
  const right = b.orderIndex ?? Number.MAX_SAFE_INTEGER;
  return left - right || a.uid.localeCompare(b.uid);
}
