import { QueryClientProvider, type QueryKey } from "@tanstack/react-query";
import { render } from "@testing-library/react";
import type { ReactNode } from "react";
import { createQueryClient } from "../app/query-client";

/** Seeded queries stay fresh, so only unseeded ones call a command on mount. */
export function renderComponent(
  ui: ReactNode,
  seed: [QueryKey, unknown][] = [],
) {
  const client = createQueryClient();
  for (const [queryKey, data] of seed) client.setQueryData(queryKey, data);
  return {
    client,
    ...render(<QueryClientProvider client={client}>{ui}</QueryClientProvider>),
  };
}
