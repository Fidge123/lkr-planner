import { useCallback, useEffect, useRef, useState } from "react";
import type { PlanningContactRecord } from "../../generated/tauri";
import {
  loadCachedDayliteContacts,
  loadDayliteContacts,
} from "../../services/daylite-contacts";
import type { PlanningGridEmployeesState } from "../page";

export function usePlanningEmployees(): PlanningGridEmployeesState {
  const requestIdRef = useRef(0);
  const [employees, setEmployees] = useState<PlanningContactRecord[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const loadEmployees = useCallback(async (forceRefresh: boolean) => {
    const id = ++requestIdRef.current;
    setIsLoading(true);

    if (!forceRefresh) {
      const cachedEmployees = await loadCachedDayliteContacts();
      if (id !== requestIdRef.current) return;
      if (cachedEmployees.length > 0) {
        setEmployees(cachedEmployees);
      }
    }

    try {
      const result = await loadDayliteContacts({ forceRefresh });
      if (id !== requestIdRef.current) return;
      setEmployees(result.contacts);
      setErrorMessage(result.errorMessage ?? null);
    } catch (error) {
      if (id !== requestIdRef.current) return;
      setErrorMessage(
        error instanceof Error
          ? error.message
          : "Die Mitarbeiter konnten nicht von Daylite geladen werden.",
      );
    } finally {
      if (id === requestIdRef.current) {
        setIsLoading(false);
      }
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
