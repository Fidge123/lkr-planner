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

interface WeekData {
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
      // Cache unconditionally, even for a superseded request: it's still valid
      // data for that week. Only the display update below is staleness-gated.
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
    void loadActiveWeek(debouncedWeekStart);
    void prefetchWeek(adjacentWeek(debouncedWeekStart, -7));
    void prefetchWeek(adjacentWeek(debouncedWeekStart, 7));
  }, [debouncedWeekStart, loadActiveWeek, prefetchWeek]);

  // macOS WKWebView (the packaged app's renderer) can skip repainting a card
  // whose DOM update landed via a Tauri IPC callback rather than a user input
  // event, leaving stale or partially overwritten pixels behind until the next
  // native-triggered repaint (e.g. hovering it). Dispatching a resize forces a
  // full repaint without changing any layout.
  // biome-ignore lint/correctness/useExhaustiveDependencies: eventsByEmployee is the repaint trigger, not a value the effect body reads.
  useEffect(() => {
    const frame = requestAnimationFrame(() =>
      window.dispatchEvent(new Event("resize")),
    );
    return () => cancelAnimationFrame(frame);
  }, [eventsByEmployee]);

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

  return useMemo(
    () => ({
      eventsByEmployee,
      errorsByEmployee,
      isLoading,
      errorMessage,
      loadedWeekStart,
      reloadAssignments,
      invalidateWeeksContaining,
    }),
    [
      eventsByEmployee,
      errorsByEmployee,
      isLoading,
      errorMessage,
      loadedWeekStart,
      reloadAssignments,
      invalidateWeeksContaining,
    ],
  );
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
