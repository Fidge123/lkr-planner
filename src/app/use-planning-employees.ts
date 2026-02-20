import { useCallback, useEffect, useState } from "react";
import type { DayliteContactRecord } from "../domain/planning";
import {
  loadCachedDayliteContactsFromStore,
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
      const cachedEmployees = await loadCachedDayliteContactsFromStore();
      if (cachedEmployees.length > 0) {
        console.info("[daylite-employees] using cached employees in hook", {
          count: cachedEmployees.length,
          sample: cachedEmployees.slice(0, 5).map((employee) => ({
            self: employee.self,
            full_name: employee.full_name ?? null,
            nickname: employee.nickname ?? null,
            category: employee.category ?? null,
          })),
        });
        setEmployees(cachedEmployees);
      }
    }

    try {
      const result = await loadDayliteContacts({ forceRefresh });
      console.info("[daylite-employees] hook load result", {
        source: result.source,
        count: result.contacts.length,
        errorMessage: result.errorMessage,
        sample: result.contacts.slice(0, 5).map((employee) => ({
          self: employee.self,
          full_name: employee.full_name ?? null,
          nickname: employee.nickname ?? null,
          category: employee.category ?? null,
        })),
      });
      setEmployees(result.contacts);
      setErrorMessage(result.errorMessage);
    } catch (error) {
      console.info("[daylite-employees] hook load failed", {
        error,
      });
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
