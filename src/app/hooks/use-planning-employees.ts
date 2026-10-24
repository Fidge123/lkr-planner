import { queryOptions, useQuery } from "@tanstack/react-query";
import {
  loadCachedDayliteContacts,
  loadDayliteContacts,
} from "../../services/daylite-contacts";
import type { PlanningGridEmployeesState } from "../page";

export const contactsQuery = queryOptions({
  queryKey: ["contacts"],
  queryFn: loadDayliteContacts,
});

/** Shows the last known employees until Daylite answers, and keeps them when it fails. */
export function usePlanningEmployees(): PlanningGridEmployeesState {
  const cached = useQuery({
    queryKey: ["cached-contacts"],
    queryFn: loadCachedDayliteContacts,
  });
  const contacts = useQuery(contactsQuery);

  return {
    employees: contacts.data ?? cached.data ?? [],
    isLoading: contacts.isFetching,
    errorMessage: contacts.error?.message ?? null,
    reloadEmployees: () => void contacts.refetch(),
  };
}
