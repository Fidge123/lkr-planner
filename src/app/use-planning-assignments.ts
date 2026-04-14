import { useCallback, useEffect, useRef, useState } from "react";
import type { CalendarCellEvent, EmployeeWeekEvents } from "../generated/tauri";
import { commands } from "../generated/tauri";

type EmployeeEvents = Record<string, CalendarCellEvent[]>;
type EmployeeErrors = Record<string, string>;

interface WeekData {
  eventsByEmployee: EmployeeEvents;
  errorsByEmployee: EmployeeErrors;
}

export interface PlanningAssignmentsState {
  eventsByEmployee: EmployeeEvents;
  /** Per-employee CalDAV fetch errors, keyed by employee reference. */
  errorsByEmployee: EmployeeErrors;
  isLoading: boolean;
  errorMessage: string | null;
  reloadAssignments: () => void;
}

export function usePlanningAssignments(
  weekStart: string,
): PlanningAssignmentsState {
  const cache = useRef<Record<string, WeekData>>({});
  const [eventsByEmployee, setEventsByEmployee] = useState<EmployeeEvents>({});
  const [errorsByEmployee, setErrorsByEmployee] = useState<EmployeeErrors>({});
  const [isLoading, setIsLoading] = useState(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  // Loads a week and updates visible state. Pass invalidate=true to bypass cache.
  const loadActiveWeek = useCallback(async (ws: string, invalidate = false) => {
    if (invalidate) {
      delete cache.current[ws];
    }

    const cached = cache.current[ws];
    if (cached) {
      setEventsByEmployee(cached.eventsByEmployee);
      setErrorsByEmployee(cached.errorsByEmployee);
      setIsLoading(false);
      setErrorMessage(null);
      return;
    }

    setIsLoading(true);
    try {
      const result = await commands.loadWeekEvents(ws);
      if (result.status === "error") {
        setErrorMessage(result.error);
        setEventsByEmployee({});
        setErrorsByEmployee({});
        return;
      }
      const data = groupResults(result.data);
      cache.current[ws] = data;
      setEventsByEmployee(data.eventsByEmployee);
      setErrorsByEmployee(data.errorsByEmployee);
      setErrorMessage(null);
    } catch (error) {
      setErrorMessage(
        error instanceof Error
          ? error.message
          : "Die Einsätze konnten nicht geladen werden.",
      );
      setEventsByEmployee({});
      setErrorsByEmployee({});
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Silently pre-warms the cache for an adjacent week.
  const prefetchWeek = useCallback(async (ws: string) => {
    if (cache.current[ws]) return;
    try {
      const result = await commands.loadWeekEvents(ws);
      if (result.status === "ok") {
        cache.current[ws] = groupResults(result.data);
      }
    } catch {
      // Silently ignore prefetch failures
    }
  }, []);

  useEffect(() => {
    void loadActiveWeek(weekStart);
    void prefetchWeek(adjacentWeek(weekStart, -7));
    void prefetchWeek(adjacentWeek(weekStart, 7));
  }, [weekStart, loadActiveWeek, prefetchWeek]);

  const reloadAssignments = useCallback(() => {
    void loadActiveWeek(weekStart, true);
  }, [weekStart, loadActiveWeek]);

  return {
    eventsByEmployee,
    errorsByEmployee,
    isLoading,
    errorMessage,
    reloadAssignments,
  };
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
  const d = new Date(weekStart);
  d.setDate(d.getDate() + offsetDays);
  return d.toISOString().slice(0, 10);
}
