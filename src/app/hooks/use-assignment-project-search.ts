import { keepPreviousData, useQuery } from "@tanstack/react-query";
import type { DayliteProjectSummary } from "../../generated/tauri";
import { searchProjectsForAssignmentPicker } from "../../services/assignment-project-picker";
import { useLeadingDebounce } from "./use-leading-debounce";

const minFilterLength = 3;
const debounceMs = 300;

export interface AssignmentProjectSearchState {
  results: DayliteProjectSummary[];
  /** True while the shown results do not belong to the typed term yet. */
  isSearching: boolean;
  errorMessage: string | null;
}

export function useAssignmentProjectSearch(
  filter: string,
): AssignmentProjectSearchState {
  const term = useLeadingDebounce(filter, debounceMs);
  const isSearchable =
    filter.length >= minFilterLength && term.length >= minFilterLength;
  const { data, error, isPending, isPlaceholderData } = useQuery({
    queryKey: ["project-search", term],
    queryFn: () => searchProjectsForAssignmentPicker(term),
    enabled: isSearchable,
    placeholderData: keepPreviousData,
    // Projects change in Daylite, so a repeated term shows its cached results and searches again.
    staleTime: 0,
  });

  const isSearching =
    filter.length >= minFilterLength &&
    (term !== filter || isPending || isPlaceholderData);

  return {
    results: isSearchable && !isSearching ? (data ?? []) : [],
    isSearching,
    errorMessage: isSearchable ? (error?.message ?? null) : null,
  };
}
