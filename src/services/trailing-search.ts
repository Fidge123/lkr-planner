import type { DayliteProjectSummary } from "../generated/tauri";

export interface TrailingSearchOptions {
  delayMs: number;
  minLength: number;
  search: (term: string) => Promise<DayliteProjectSummary[]>;
  onResults: (results: DayliteProjectSummary[]) => void;
  onError: (message: string) => void;
}

export interface TrailingSearch {
  setFilter: (filter: string) => void;
  dispose: () => void;
}

// Trailing-edge debounced search with a monotonic request guard for
// search-as-you-type. Each filter change restarts the debounce timer, so only
// the settled term is queried; a request id ensures a slow earlier response
// cannot overwrite a newer one. Below `minLength` the result list is cleared
// and any in-flight query is invalidated, returning to the empty default state.
export function createTrailingSearch(
  options: TrailingSearchOptions,
): TrailingSearch {
  let timer: ReturnType<typeof setTimeout> | null = null;
  let requestSeq = 0;

  return {
    setFilter(filter: string): void {
      if (timer) clearTimeout(timer);

      if (filter.length < options.minLength) {
        requestSeq++;
        options.onResults([]);
        return;
      }

      timer = setTimeout(() => {
        const id = ++requestSeq;
        options.search(filter).then(
          (results) => {
            if (id === requestSeq) options.onResults(results);
          },
          (error: unknown) => {
            if (id !== requestSeq) return;
            options.onError(
              error instanceof Error
                ? error.message
                : "Projekte konnten nicht geladen werden.",
            );
          },
        );
      }, options.delayMs);
    },
    dispose(): void {
      if (timer) clearTimeout(timer);
    },
  };
}
