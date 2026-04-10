import { useCallback, useEffect, useState } from "react";
import type { CalendarCellEvent } from "../generated/tauri";
import { commands } from "../generated/tauri";

export interface PlanningAssignmentsState {
  eventsByEmployee: Record<string, CalendarCellEvent[]>;
  isLoading: boolean;
  errorMessage: string | null;
  reloadAssignments: () => void;
}

export function usePlanningAssignments(weekStart: string): PlanningAssignmentsState {
  const [eventsByEmployee, setEventsByEmployee] = useState<
    Record<string, CalendarCellEvent[]>
  >({});
  const [isLoading, setIsLoading] = useState(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const loadAssignments = useCallback(async () => {
    setIsLoading(true);

    try {
      const result = await commands.loadWeekEvents(weekStart);
      if (result.status === "error") {
        setErrorMessage(result.error);
        setEventsByEmployee({});
        return;
      }

      const grouped: Record<string, CalendarCellEvent[]> = {};
      for (const entry of result.data) {
        if (!entry.error) {
          grouped[entry.employeeReference] = entry.events;
        }
      }
      setEventsByEmployee(grouped);
      setErrorMessage(null);
    } catch (error) {
      setErrorMessage(
        error instanceof Error
          ? error.message
          : "Die Einsätze konnten nicht geladen werden.",
      );
      setEventsByEmployee({});
    } finally {
      setIsLoading(false);
    }
  }, [weekStart]);

  useEffect(() => {
    void loadAssignments();
  }, [loadAssignments]);

  const reloadAssignments = useCallback(() => {
    void loadAssignments();
  }, [loadAssignments]);

  return { eventsByEmployee, isLoading, errorMessage, reloadAssignments };
}
