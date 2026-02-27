import { useCallback, useEffect, useState } from "react";
import type { DayliteContactRecord } from "../domain/planning";
import {
  loadCachedDayliteContacts,
  loadDayliteContacts,
} from "../services/daylite-contacts";
import type { PlanningGridEmployeesState } from "./page";

export function usePlanningEmployees(): PlanningGridEmployeesState {
  const [employees, setEmployees] = useState<DayliteContactRecord[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const loadEmployees = useCallback(async (forceRefresh: boolean) => {
    setIsLoading(true);

    if (!forceRefresh) {
      const cachedEmployees = await loadCachedDayliteContacts();
      if (cachedEmployees.length > 0) {
        setEmployees(cachedEmployees);
      }
    }

    try {
      const result = await loadDayliteContacts({ forceRefresh });
      setEmployees(result.contacts);
      setErrorMessage(result.errorMessage ?? null);
    } catch (error) {
      setErrorMessage(
        error instanceof Error
          ? error.message
          : "Die Mitarbeiter konnten nicht von Daylite geladen werden.",
      );
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadEmployees(false);
  }, [loadEmployees]);

  const reloadEmployees = useCallback(() => {
    void loadEmployees(true);
  }, [loadEmployees]);

  return { employees, isLoading, errorMessage, reloadEmployees };
}
