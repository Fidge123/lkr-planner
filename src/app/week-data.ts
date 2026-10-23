import type {
  CalendarCellEvent,
  EmployeeWeekEvents,
  ResolvedProject,
} from "../generated/tauri";
import { sortCellEvents } from "./cell-order";
import { isProtectedAssignment } from "./components/assignment-modal-logic";

export interface WeekData {
  eventsByEmployee: Record<string, CalendarCellEvent[]>;
  errorsByEmployee: Record<string, string>;
}

export interface CellTarget {
  employeeRef: string;
  date: string;
  /** Position among the target cell's assignments, counted without the moved event. */
  orderIndex: number;
}

export function groupResults(entries: EmployeeWeekEvents[]): WeekData {
  const eventsByEmployee: WeekData["eventsByEmployee"] = {};
  const errorsByEmployee: WeekData["errorsByEmployee"] = {};
  for (const entry of entries) {
    if (entry.error) {
      errorsByEmployee[entry.employeeReference] = entry.error;
    } else {
      eventsByEmployee[entry.employeeReference] = entry.events;
    }
  }
  return { eventsByEmployee, errorsByEmployee };
}

export function projectRefsOf(week: WeekData): string[] {
  const refs = new Set<string>();
  for (const events of Object.values(week.eventsByEmployee)) {
    for (const event of events) {
      if (event.kind === "assignment" && event.projectRef !== null) {
        refs.add(event.projectRef);
      }
    }
  }
  return [...refs];
}

/** A `null` project leaves the assignment unresolved, but no longer pending. */
export function applyResolvedProjects(
  week: WeekData,
  resolved: Partial<Record<string, ResolvedProject | null>>,
): WeekData {
  return mapEvents(week, (event) => {
    if (event.projectRef === null || !(event.projectRef in resolved)) {
      return event;
    }
    const project = resolved[event.projectRef];
    if (!project) {
      return event.projectPending ? { ...event, projectPending: false } : event;
    }
    const unchanged =
      !event.projectPending &&
      event.title === project.name &&
      event.projectStatus === project.status &&
      event.projectCategory === project.category;
    return unchanged
      ? event
      : {
          ...event,
          title: project.name,
          projectStatus: project.status,
          projectCategory: project.category,
          projectPending: false,
        };
  });
}

export function settlePendingProjects(
  week: WeekData,
  projectRefs: string[],
): WeekData {
  const settled = new Set(projectRefs);
  return mapEvents(week, (event) =>
    event.projectPending &&
    event.projectRef !== null &&
    settled.has(event.projectRef)
      ? { ...event, projectPending: false }
      : event,
  );
}

/** Every requested employee is replaced, so one the backend no longer returns drops out. */
export function mergeEmployeeRows(
  week: WeekData,
  employeeRefs: string[],
  entries: EmployeeWeekEvents[],
): WeekData {
  const eventsByEmployee = { ...week.eventsByEmployee };
  const errorsByEmployee = { ...week.errorsByEmployee };
  for (const ref of employeeRefs) {
    delete eventsByEmployee[ref];
    delete errorsByEmployee[ref];
  }
  const refreshed = groupResults(entries);
  return {
    eventsByEmployee: { ...eventsByEmployee, ...refreshed.eventsByEmployee },
    errorsByEmployee: { ...errorsByEmployee, ...refreshed.errorsByEmployee },
  };
}

export function findEvent(
  week: WeekData,
  uid: string,
): CalendarCellEvent | null {
  for (const events of Object.values(week.eventsByEmployee)) {
    const found = events.find((event) => event.uid === uid);
    if (found) return found;
  }
  return null;
}

export function removeEvent(week: WeekData, uid: string): WeekData {
  const eventsByEmployee: WeekData["eventsByEmployee"] = {};
  for (const [ref, events] of Object.entries(week.eventsByEmployee)) {
    eventsByEmployee[ref] = events.filter((event) => event.uid !== uid);
  }
  return { ...week, eventsByEmployee };
}

/**
 * Optimistic placement; times are cleared because the backend re-slots the day.
 * A fixed appointment dropped on another day stays put, as the backend refuses that move.
 */
export function relocateEvent(
  week: WeekData,
  event: CalendarCellEvent,
  target: CellTarget,
): WeekData {
  if (
    isProtectedAssignment(event.projectCategory) &&
    event.date !== target.date
  ) {
    return week;
  }
  const without = removeEvent(week, event.uid);
  const targetEvents = without.eventsByEmployee[target.employeeRef] ?? [];
  const isTargetAssignment = (candidate: CalendarCellEvent) =>
    candidate.kind === "assignment" && candidate.date === target.date;

  const cell = sortCellEvents(targetEvents.filter(isTargetAssignment));
  const index = Math.min(Math.max(target.orderIndex, 0), cell.length);
  cell.splice(index, 0, {
    ...event,
    date: target.date,
    startTime: null,
    endTime: null,
  });

  return {
    ...without,
    eventsByEmployee: {
      ...without.eventsByEmployee,
      [target.employeeRef]: [
        ...targetEvents.filter((candidate) => !isTargetAssignment(candidate)),
        ...cell.map((assignment, orderIndex) => ({
          ...assignment,
          orderIndex,
        })),
      ],
    },
  };
}

/** Tells a refresh whether an optimistic edit happened after it was requested. */
export function createEditSequence() {
  let edits = 0;
  return {
    markEdit: () => {
      edits += 1;
    },
    requestRefresh: () => {
      const requestedAt = edits;
      return () => edits === requestedAt;
    },
  };
}

/** Hands back the same week when no event changed, so an idle patch causes no re-render. */
function mapEvents(
  week: WeekData,
  transform: (event: CalendarCellEvent) => CalendarCellEvent,
): WeekData {
  let changed = false;
  const eventsByEmployee: WeekData["eventsByEmployee"] = {};
  for (const [ref, events] of Object.entries(week.eventsByEmployee)) {
    eventsByEmployee[ref] = events.map((event) => {
      const next = transform(event);
      changed ||= next !== event;
      return next;
    });
  }
  return changed ? { ...week, eventsByEmployee } : week;
}
