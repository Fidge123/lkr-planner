import { useEffect, useState } from "react";
import type { DayliteProjectSummary } from "../../generated/tauri";
import { loadDefaultSuggestions } from "../../services/assignment-suggestions";

export interface AssignmentDefaultSuggestionsState {
  suggestions: DayliteProjectSummary[];
  suggestionsLoaded: boolean;
}

export function useAssignmentDefaultSuggestions(
  isOpen: boolean,
): AssignmentDefaultSuggestionsState {
  const [suggestions, setSuggestions] = useState<DayliteProjectSummary[]>([]);
  const [suggestionsLoaded, setSuggestionsLoaded] = useState(false);

  useEffect(() => {
    if (!isOpen) return;
    let cancelled = false;
    setSuggestions([]);
    setSuggestionsLoaded(false);
    loadDefaultSuggestions().then((next) => {
      if (cancelled) return;
      setSuggestions(next);
      setSuggestionsLoaded(true);
    });
    return () => {
      cancelled = true;
    };
  }, [isOpen]);

  return { suggestions, suggestionsLoaded };
}
