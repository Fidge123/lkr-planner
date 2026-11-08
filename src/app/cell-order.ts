import type { CellEvent } from "./types";

export function assignmentPositions(events: CellEvent[]): Map<string, number> {
  const positions = new Map<string, number>();
  for (const event of events) {
    if (event.kind === "assignment") {
      positions.set(event.uid, positions.size);
    }
  }
  return positions;
}
