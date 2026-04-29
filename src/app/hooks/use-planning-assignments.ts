import { useCallback, useEffect, useRef, useState } from "react";
import type {
  CalendarCellEvent,
  EmployeeWeekEvents,
} from "../../generated/tauri";
import { commands } from "../../generated/tauri";
import { useLeadingDebounce } from "./use-leading-debounce";
import { toLocalISODate } from "../util";

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
  reloadAssignments: () => void;
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

  // Loads a week and updates visible state. Pass invalidate=true to bypass cache.
  const loadActiveWeek = useCallback(async (ws: string, invalidate = false) => {
    if (invalidate) {
      delete cache.current[ws];
    }

    const id = ++requestIdRef.current;

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
      if (id !== requestIdRef.current) return;
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
      if (id !== requestIdRef.current) return;
      setErrorMessage(
        error instanceof Error
          ? error.message
          : "Die Einsätze konnten nicht geladen werden.",
      );
      setEventsByEmployee({});
      setErrorsByEmployee({});
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
    } catch {
      // Silently ignore prefetch failures
    }
  }, []);

  // Debounced: load active week and prefetch adjacent weeks after navigation settles
  useEffect(() => {
    void loadActiveWeek(debouncedWeekStart);
    void prefetchWeek(adjacentWeek(debouncedWeekStart, -7));
    void prefetchWeek(adjacentWeek(debouncedWeekStart, 7));
  }, [debouncedWeekStart, loadActiveWeek, prefetchWeek]);

  const reloadAssignments = useCallback(() => {
    cache.current = {};
    void loadActiveWeek(weekStart);
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
  const [y, m, d] = weekStart.split("-").map(Number);
  return toLocalISODate(new Date(y, m - 1, d + offsetDays));
}
