import { useQuery } from "@tanstack/react-query";
import type { DayliteProjectSummary } from "../../generated/tauri";
import {
  combineSuggestions,
  getLastAssignedProject,
  loadOverdueProjects,
} from "../../services/assignment-suggestions";

export interface AssignmentDefaultSuggestionsState {
  suggestions: DayliteProjectSummary[];
  suggestionsLoaded: boolean;
}

export function useAssignmentDefaultSuggestions(): AssignmentDefaultSuggestionsState {
  const overdue = useQuery({
    queryKey: ["overdue-projects"],
    queryFn: loadOverdueProjects,
    // Projects fall overdue as days pass, so every open refreshes the shown list.
    staleTime: 0,
  });
  return {
    suggestions: combineSuggestions(
      getLastAssignedProject(),
      overdue.data ?? [],
    ),
    suggestionsLoaded: !overdue.isPending,
  };
}
