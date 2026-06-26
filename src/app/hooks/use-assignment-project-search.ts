import { useEffect, useRef, useState } from "react";
import type { DayliteProjectSummary } from "../../generated/tauri";
import { searchProjectsForAssignmentPicker } from "../../services/assignment-project-picker";
import {
  createTrailingSearch,
  type TrailingSearch,
} from "../../services/trailing-search";

const MIN_FILTER_LENGTH = 3;
const DEBOUNCE_MS = 300;

export interface AssignmentProjectSearchState {
  results: DayliteProjectSummary[];
  errorMessage: string | null;
}

// React adapter over `createTrailingSearch`: feeds the filter into the trailing
// debounced search and exposes its results. Querying, debouncing and stale
// response dropping live in the controller; this hook only bridges to state.
export function useAssignmentProjectSearch(
  filter: string,
): AssignmentProjectSearchState {
  const [results, setResults] = useState<DayliteProjectSummary[]>([]);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const controllerRef = useRef<TrailingSearch | null>(null);

  if (controllerRef.current === null) {
    controllerRef.current = createTrailingSearch({
      delayMs: DEBOUNCE_MS,
      minLength: MIN_FILTER_LENGTH,
      search: searchProjectsForAssignmentPicker,
      onResults: (next) => {
        setResults(next);
        setErrorMessage(null);
      },
      onError: (message) => {
        setResults([]);
        setErrorMessage(message);
      },
    });
  }

  useEffect(() => {
    controllerRef.current?.setFilter(filter);
  }, [filter]);

  useEffect(() => () => controllerRef.current?.dispose(), []);

  return { results, errorMessage };
}
