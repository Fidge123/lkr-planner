import { queryOptions, useQuery } from "@tanstack/react-query";
import { loadZepCredentials } from "../../services/zep";

export const zepCredentialsQuery = queryOptions({
  queryKey: ["zep-credentials"],
  queryFn: loadZepCredentials,
});

export function useZepCredentials() {
  const { data, error } = useQuery(zepCredentialsQuery);
  return { credentials: data ?? null, errorMessage: error?.message ?? null };
}
