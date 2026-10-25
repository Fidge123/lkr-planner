import { queryOptions, useQuery } from "@tanstack/react-query";
import { displaySettingsOf } from "../../services/display-settings";
import { loadLocalStore } from "../../services/local-store";

export const localStoreQuery = queryOptions({
  queryKey: ["local-store"],
  queryFn: loadLocalStore,
});

export function useLocalStore() {
  const { data, error, isPending } = useQuery(localStoreQuery);
  return {
    isLoaded: !isPending,
    errorMessage: error?.message ?? null,
    employeeSettings: data?.employeeSettings ?? [],
    displaySettings: displaySettingsOf(data),
  };
}
