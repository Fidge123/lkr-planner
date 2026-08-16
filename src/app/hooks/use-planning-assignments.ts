import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type {
  CalendarCellEvent,
  EmployeeWeekEvents,
} from "../../generated/tauri";
import { commands } from "../../generated/tauri";
import { toLocalISODate } from "../util";
import { useLeadingDebounce } from "./use-leading-debounce";

type EmployeeEvents = Record<string, CalendarCellEvent[]>;
type EmployeeErrors = Record<string, string>;

export interface WeekData {
  eventsByEmployee: EmployeeEvents;
  errorsByEmployee: EmployeeErrors;
}

export interface PlanningAssignmentsState {
  eventsByEmployee: EmployeeEvents;
  errorsByEmployee: EmployeeErrors;
  isLoading: boolean;
  errorMessage: string | null;
  loadedWeekStart: string | null;
  reloadAssignments: () => void;
  invalidateWeeksContaining: (uid: string) => void;
  getCachedWeek: (weekStart: string) => WeekData | null;
}

export function usePlanningAssignments(
  weekStart: string,
): PlanningAssignmentsState {
  const debouncedWeekStart = useLeadingDebounce(weekStart, 200);
  const cache = useRef<Record<string, WeekData>>({});
  const requestIdRef = useRef(0);
  const [eventsByEmployee, setEventsByEmployee] = useState<EmployeeEvents>({});
  const [errorsByEmployee, setErrorsByEmployee] = useState<EmployeeErrors>({});
  const [isLoading, setIsLoading] = useState(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  // Set in the same batch as the events, so a consumer can tell whether it is
  // rendering this week's data or the previous week's, still up during a fetch.
  const [loadedWeekStart, setLoadedWeekStart] = useState<string | null>(null);

  const loadActiveWeek = useCallback(async (ws: string, invalidate = false) => {
    if (invalidate) {
      delete cache.current[ws];
    }

    const id = ++requestIdRef.current;

    const cached = cache.current[ws];
    if (cached) {
      setEventsByEmployee(cached.eventsByEmployee);
      setErrorsByEmployee(cached.errorsByEmployee);
      setLoadedWeekStart(ws);
      setIsLoading(false);
      setErrorMessage(null);
      return;
    }

    setIsLoading(true);
    try {
      const result = await commands.loadWeekEvents(ws);
      if (result.status === "error") {
        if (id !== requestIdRef.current) return;
        setErrorMessage(result.error);
        setEventsByEmployee({});
        setErrorsByEmployee({});
        setLoadedWeekStart(ws);
        return;
      }
      // Cache unconditionally, even for a superseded request: it's still valid data for that week.
      // Only the display update below is staleness-gated.
      const data = groupResults(result.data);
      cache.current[ws] = data;
      if (id !== requestIdRef.current) return;
      setEventsByEmployee(data.eventsByEmployee);
      setErrorsByEmployee(data.errorsByEmployee);
      setLoadedWeekStart(ws);
      setErrorMessage(null);
    } catch (error) {
      if (id !== requestIdRef.current) return;
      setErrorMessage(
        error instanceof Error
          ? error.message
          : "Die Einsätze konnten nicht geladen werden.",
      );
      setEventsByEmployee({});
      setErrorsByEmployee({});
      setLoadedWeekStart(ws);
    } finally {
      if (id === requestIdRef.current) {
        setIsLoading(false);
      }
    }
  }, []);

  const prefetchWeek = useCallback(async (ws: string) => {
    if (cache.current[ws]) return;
    try {
      const result = await commands.loadWeekEvents(ws);
      if (result.status === "ok") {
        cache.current[ws] = groupResults(result.data);
      }
    } catch {}
  }, []);

  useEffect(() => {
    let cancelled = false;
    let idleTimer: ReturnType<typeof setTimeout> | undefined;
    let settleIdle: (() => void) | undefined;
    void loadWeekWithPrefetch({
      weekStart: debouncedWeekStart,
      loadActive: loadActiveWeek,
      prefetch: prefetchWeek,
      isCancelled: () => cancelled,
      wait: (ms) =>
        new Promise((resolve) => {
          settleIdle = resolve;
          idleTimer = setTimeout(resolve, ms);
        }),
    });
    return () => {
      cancelled = true;
      clearTimeout(idleTimer);
      // Clearing the timer alone leaves the sequence suspended on a promise nothing resolves.
      settleIdle?.();
    };
  }, [debouncedWeekStart, loadActiveWeek, prefetchWeek]);

  const reloadAssignments = useCallback(() => {
    void loadActiveWeek(weekStart, true);
  }, [weekStart, loadActiveWeek]);

  // A drag can end in a different week than it started, and reloadAssignments
  // only refreshes the week active on drop, leaving the source week cached with
  // the event still in its old slot.
  const invalidateWeeksContaining = useCallback((uid: string) => {
    for (const [ws, data] of Object.entries(cache.current)) {
      const holdsUid = Object.values(data.eventsByEmployee).some((events) =>
        events.some((event) => event.uid === uid),
      );
      if (holdsUid) {
        delete cache.current[ws];
      }
    }
  }, []);

  const getCachedWeek = useCallback(
    (ws: string) => cache.current[ws] ?? null,
    [],
  );

  return useMemo(
    () => ({
      eventsByEmployee,
      errorsByEmployee,
      isLoading,
      errorMessage,
      loadedWeekStart,
      reloadAssignments,
      invalidateWeeksContaining,
      getCachedWeek,
    }),
    [
      eventsByEmployee,
      errorsByEmployee,
      isLoading,
      errorMessage,
      loadedWeekStart,
      reloadAssignments,
      invalidateWeeksContaining,
      getCachedWeek,
    ],
  );
}

export interface WeekLoadSequence {
  weekStart: string;
  loadActive: (weekStart: string) => Promise<void>;
  prefetch: (weekStart: string) => Promise<void>;
  isCancelled: () => boolean;
  wait: (ms: number) => Promise<void>;
}

const prefetchIdleMs = 400;

/**
 * A prefetch competes with the active week for CalDAV connections and Daylite requests.
 * A started week load cannot be called off, so it waits for the user to settle first.
 */
export async function loadWeekWithPrefetch({
  weekStart,
  loadActive,
  prefetch,
  isCancelled,
  wait,
}: WeekLoadSequence): Promise<void> {
  await loadActive(weekStart);
  if (isCancelled()) return;

  await wait(prefetchIdleMs);
  if (isCancelled()) return;

  await Promise.all([
    prefetch(adjacentWeek(weekStart, -7)),
    prefetch(adjacentWeek(weekStart, 7)),
  ]);
}

function groupResults(entries: EmployeeWeekEvents[]): WeekData {
  const eventsByEmployee: EmployeeEvents = {};
  const errorsByEmployee: EmployeeErrors = {};
  for (const entry of entries) {
    if (entry.error) {
      errorsByEmployee[entry.employeeReference] = entry.error;
    } else {
      eventsByEmployee[entry.employeeReference] = entry.events;
    }
  }
  return { eventsByEmployee, errorsByEmployee };
}

function adjacentWeek(weekStart: string, offsetDays: number): string {
  const [y, m, d] = weekStart.split("-").map(Number);
  return toLocalISODate(new Date(y, m - 1, d + offsetDays));
}
