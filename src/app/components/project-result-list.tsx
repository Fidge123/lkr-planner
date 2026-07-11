import { useEffect, useRef } from "react";
import type { DayliteProjectSummary } from "../../generated/tauri";

export function ProjectResultList({
  projects,
  highlightedIndex,
  onSelect,
}: ProjectResultListProps) {
  const activeRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (highlightedIndex < 0) return;
    activeRef.current?.scrollIntoView({ block: "nearest" });
  }, [highlightedIndex]);

  if (projects.length === 0) return null;

  return (
    <ul
      id="assignment-project-results"
      className="menu menu-sm bg-base-200 rounded-box w-full p-1"
    >
      {projects.map((project, index) => {
        const isActive = index === highlightedIndex;
        return (
          <li key={project.self}>
            <button
              ref={isActive ? activeRef : undefined}
              type="button"
              aria-current={isActive}
              className={
                isActive ? "bg-primary text-primary-content" : undefined
              }
              onClick={() => onSelect(project)}
            >
              {project.name}
            </button>
          </li>
        );
      })}
    </ul>
  );
}

interface ProjectResultListProps {
  projects: DayliteProjectSummary[];
  highlightedIndex: number;
  onSelect: (project: DayliteProjectSummary) => void;
}

export function SuggestionEmptyState({
  filter,
  suggestionsLoaded,
  suggestionCount,
}: SuggestionEmptyStateProps) {
  if (filter.length > 0 || !suggestionsLoaded || suggestionCount > 0) {
    return null;
  }
  return <p className="text-sm">Keine Vorschläge verfügbar</p>;
}

interface SuggestionEmptyStateProps {
  filter: string;
  suggestionsLoaded: boolean;
  suggestionCount: number;
}
