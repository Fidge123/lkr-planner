import { useEffect, useRef } from "react";
import type { DayliteProjectSummary } from "../../generated/tauri";
import {
  type ProjectCategoryColors,
  projectCategoryColor,
} from "../../services/daylite-categories";
import { assignmentStripClass, categoryStrip } from "./timetable-cell";

export function ProjectResultList({
  projects,
  categoryColors = {},
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
              className={`${assignmentStripClass} rounded-lg ${isActive ? "bg-primary text-primary-content" : ""}`}
              style={categoryStrip(
                projectCategoryColor(categoryColors, project.category),
              )}
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
  /** Daylite category name to color, so a result carries the same strip as its cards. */
  categoryColors?: ProjectCategoryColors;
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
