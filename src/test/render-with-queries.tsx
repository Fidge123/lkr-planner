import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";

/** A static render runs no effects, so its queries never fetch. */
export function renderWithQueries(ui: ReactNode): string {
  return renderToStaticMarkup(
    <QueryClientProvider client={new QueryClient()}>{ui}</QueryClientProvider>,
  );
}
