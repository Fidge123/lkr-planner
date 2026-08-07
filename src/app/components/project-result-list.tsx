import { useEffect, useRef } from "react";
import type { DayliteProjectSummary } from "../../generated/tauri";
import {
  type ProjectCategoryColors,
  projectCategoryColor,
} from "../../services/daylite-categories";

const resultListClass =
  "bg-base-200 rounded-box w-full p-1 max-h-64 overflow-y-auto list-none";

// The fixed height keeps a loaded row and its placeholder the same size.
const resultRowClass =
  "flex items-center gap-2 w-full h-8 px-2 rounded-md text-left text-sm";

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
    <ul id="assignment-project-results" className={resultListClass}>
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
              className={`${resultRowClass} transition-colors ${isActive ? "bg-primary text-primary-content" : "hover:bg-base-300"}`}
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

/** Matches the suggestion limit in `assignment-suggestions`. */
const placeholderRows = [0, 1, 2, 3, 4];

export function ProjectResultPlaceholder() {
  return (
    <ul className={`${resultListClass} pointer-events-none`} aria-hidden="true">
      {placeholderRows.map((row) => (
        <li key={row} className={`${resultRowClass} opacity-60`}>
          <span className="size-2 shrink-0 rounded-full bg-base-content/30" />
          <span className="skeleton h-3 flex-1 min-w-0" />
        </li>
      ))}
    </ul>
  );
}

interface ProjectResultListProps {
  projects: DayliteProjectSummary[];
  categoryColors?: ProjectCategoryColors;
  highlightedIndex: number;
  onSelect: (project: DayliteProjectSummary) => void;
}

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
