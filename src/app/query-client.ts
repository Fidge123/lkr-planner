import { QueryClient } from "@tanstack/react-query";

/** Data stays fresh until a write or "Daten neu laden" invalidates it, and a failure surfaces at once with its reload action. */
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: Number.POSITIVE_INFINITY,
      retry: false,
      refetchOnWindowFocus: false,
      refetchOnReconnect: false,
    },
  },
});
