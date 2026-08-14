import { useEffect, useState } from "react";
import type { DayliteProjectSummary } from "../../generated/tauri";
import { loadDefaultSuggestions } from "../../services/assignment-suggestions";

export interface AssignmentDefaultSuggestionsState {
  suggestions: DayliteProjectSummary[];
  suggestionsLoaded: boolean;
}

export function useAssignmentDefaultSuggestions(): AssignmentDefaultSuggestionsState {
  const [suggestions, setSuggestions] = useState<DayliteProjectSummary[]>([]);
  const [suggestionsLoaded, setSuggestionsLoaded] = useState(false);

  useEffect(() => {
    let cancelled = false;
    loadDefaultSuggestions().then((next) => {
      if (cancelled) return;
      setSuggestions(next);
      setSuggestionsLoaded(true);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  return { suggestions, suggestionsLoaded };
}
