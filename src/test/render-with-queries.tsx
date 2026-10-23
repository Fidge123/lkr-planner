import {
  QueryClient,
  QueryClientProvider,
  type QueryKey,
} from "@tanstack/react-query";
import type { ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";

/** A static render runs no effects, so its queries never fetch and show only the seeded data. */
export function renderWithQueries(
  ui: ReactNode,
  seed: [QueryKey, unknown][] = [],
): string {
  const client = new QueryClient();
  for (const [queryKey, data] of seed) client.setQueryData(queryKey, data);
  return renderToStaticMarkup(
    <QueryClientProvider client={client}>{ui}</QueryClientProvider>,
  );
}
