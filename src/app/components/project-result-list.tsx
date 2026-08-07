import { useEffect, useRef } from "react";
import type { DayliteProjectSummary } from "../../generated/tauri";
import {
  type ProjectCategoryColors,
  projectCategoryColor,
} from "../../services/daylite-categories";

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
      className="bg-base-200 rounded-box w-full p-1 max-h-64 overflow-y-auto list-none"
    >
      {projects.map((project, index) => {
        const isActive = index === highlightedIndex;
        const name = singleLine(project.name);
        return (
          <li key={project.self}>
            <button
              ref={isActive ? activeRef : undefined}
              type="button"
              aria-current={isActive}
              title={name}
              className={`flex items-center gap-2 w-full px-2 py-1.5 rounded-md text-left text-sm transition-colors ${isActive ? "bg-primary text-primary-content" : "hover:bg-base-300"}`}
              onClick={() => onSelect(project)}
            >
              <CategoryDot
                color={projectCategoryColor(categoryColors, project.category)}
              />
              <span className="flex-1 min-w-0 truncate">{name}</span>
            </button>
          </li>
        );
      })}
    </ul>
  );
}

interface ProjectResultListProps {
  projects: DayliteProjectSummary[];
  /** Daylite category name to color, so a result carries its category at a glance. */
  categoryColors?: ProjectCategoryColors;
  highlightedIndex: number;
  onSelect: (project: DayliteProjectSummary) => void;
}

/** Keeps its own color on a selected row, where the row's text turns inverted. */
function CategoryDot({ color }: { color: string | null }) {
  return (
    <span
      aria-hidden="true"
      className={`size-2 shrink-0 rounded-full ${color ? "" : "bg-base-content/30"}`}
      style={color ? { backgroundColor: color } : undefined}
    />
  );
}

// Daylite project names carry hard line breaks, which would turn a row into a
// ragged two-line block.
function singleLine(name: string): string {
  return name.replace(/\s+/g, " ").trim();
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
