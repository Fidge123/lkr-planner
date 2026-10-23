import type { CellEvent } from "./types";

type OrderedEvent = Pick<
  CellEvent,
  "uid" | "kind" | "orderIndex" | "startTime" | "endTime"
>;

/**
 * Orders assignment cards like the backend's `ordering_key`, so visuals match allocated slots.
 * Absences and bare events keep the positions the backend gave them.
 */
export function sortCellEvents<T extends OrderedEvent>(events: T[]): T[] {
  const assignments = events
    .filter((event) => event.kind === "assignment")
    .sort(byCellOrder);
  let next = 0;
  return events.map((event) =>
    event.kind === "assignment" ? assignments[next++] : event,
  );
}

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
 * Ties on the persisted index fall back to earlier start, longer duration, then UID.
 * Unindexed and untimed assignments sort last.
 */
function byCellOrder(a: OrderedEvent, b: OrderedEvent): number {
  return (
    orderIndexOf(a) - orderIndexOf(b) ||
    startMinutesOf(a) - startMinutesOf(b) ||
    durationMinutesOf(b) - durationMinutesOf(a) ||
    // Code-unit order, matching the backend's bytewise comparison of the UID.
    (a.uid < b.uid ? -1 : a.uid > b.uid ? 1 : 0)
  );
}

function orderIndexOf(event: OrderedEvent): number {
  return event.orderIndex ?? Number.MAX_SAFE_INTEGER;
}

function startMinutesOf(event: OrderedEvent): number {
  return minutesOfDay(event.startTime) ?? Number.MAX_SAFE_INTEGER;
}

function durationMinutesOf(event: OrderedEvent): number {
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
