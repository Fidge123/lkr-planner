import { commands, type DayliteProjectSummary } from "../generated/tauri";

const SUGGESTION_LIMIT = 5;

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

export function combineSuggestions(
  recent: DayliteProjectSummary | null,
  overdue: DayliteProjectSummary[],
): DayliteProjectSummary[] {
  const suggestions = recent
    ? [recent, ...overdue.filter((project) => project.self !== recent.self)]
    : overdue;
  return suggestions.slice(0, SUGGESTION_LIMIT);
}

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
