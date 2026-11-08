import { commands, type DayliteProjectSummary } from "../generated/tauri";
import { callCommand } from "./command-result";

const suggestionLimit = 5;

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
  return suggestions.slice(0, suggestionLimit);
}

export function loadOverdueProjects(): Promise<DayliteProjectSummary[]> {
  return callCommand(
    commands.dayliteQueryOverdueProjects(),
    "Die überfälligen Projekte konnten nicht geladen werden.",
  );
}
