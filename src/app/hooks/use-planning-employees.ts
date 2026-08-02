import {
  loadDayliteContacts,
  preloadDayliteContactsFromDisk,
} from "../../services/daylite-contacts";
import type { PlanningGridEmployeesState } from "../page";
import { useCachedResource } from "./use-cached-resource";

export function usePlanningEmployees(): PlanningGridEmployeesState {
  const { data, isLoading, errorMessage, reload } = useCachedResource({
    load: loadDayliteContacts,
    errorMessage: "Die Mitarbeiter konnten nicht von Daylite geladen werden.",
    preload: preloadDayliteContactsFromDisk,
  });

  return {
    employees: data,
    isLoading,
    errorMessage,
    reloadEmployees: reload,
  };
}
