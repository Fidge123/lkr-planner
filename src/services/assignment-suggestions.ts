import { commands, type DayliteProjectSummary } from "../generated/tauri";

const SUGGESTION_LIMIT = 5;

// Last project assigned during this session. In-memory by design: it resets on
// app restart, so a fresh session shows overdue-only suggestions.
let lastAssignedProject: DayliteProjectSummary | null = null;

export function recordLastAssignedProject(
  project: DayliteProjectSummary,
): void {
  lastAssignedProject = project;
}

export function getLastAssignedProject(): DayliteProjectSummary | null {
  return lastAssignedProject;
}

export function resetLastAssignedProject(): void {
  lastAssignedProject = null;
}

// Recent project first, then the overdue projects without the recent
// duplicate. Dedup runs before the cap, so the list holds 5 distinct projects
// when that many exist.
export function combineSuggestions(
  recent: DayliteProjectSummary | null,
  overdue: DayliteProjectSummary[],
): DayliteProjectSummary[] {
  const suggestions = recent
    ? [recent, ...overdue.filter((project) => project.self !== recent.self)]
    : overdue;
  return suggestions.slice(0, SUGGESTION_LIMIT);
}

// Default suggestions for the assignment modal: the cached recent project plus
// overdue projects. A failing overdue query degrades to the cached project (or
// the empty state) instead of blocking the modal.
export async function loadDefaultSuggestions(): Promise<
  DayliteProjectSummary[]
> {
  return combineSuggestions(getLastAssignedProject(), await queryOverdue());
}

async function queryOverdue(): Promise<DayliteProjectSummary[]> {
  const result = await commands.dayliteQueryOverdueProjects().catch(() => null);
  if (!result || result.status === "error") return [];
  return result.data;
}
