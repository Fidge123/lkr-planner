import { commands, type DayliteProjectSummary } from "../generated/tauri";
import { callCommand } from "./command-result";

const displayLimit = 5;
const candidateLimit = 50;

export async function searchProjectsForAssignmentPicker(
  searchTerm: string,
): Promise<DayliteProjectSummary[]> {
  const result = await callCommand(
    commands.dayliteSearchProjects({
      searchTerm,
      limit: candidateLimit,
      statuses: ["new_status", "in_progress"],
      fullRecords: true,
      start: null,
      sort: "name",
    }),
    "Projekte konnten nicht geladen werden.",
  );

  return (result.results ?? []).slice(0, displayLimit);
}
