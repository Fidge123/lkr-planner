import { commands, type DayliteProjectSummary } from "../generated/tauri";
import { unwrapCommandResult } from "./command-result";

const DISPLAY_LIMIT = 5;
const CANDIDATE_LIMIT = 50;

export async function searchProjectsForAssignmentPicker(
  searchTerm: string,
): Promise<DayliteProjectSummary[]> {
  const result = unwrapCommandResult(
    await commands.dayliteSearchProjects({
      searchTerm,
      limit: CANDIDATE_LIMIT,
      statuses: ["new_status", "in_progress"],
      fullRecords: null,
      start: null,
      sort: "name",
    }),
    "Projekte konnten nicht geladen werden.",
  );

  return (result.results ?? []).slice(0, DISPLAY_LIMIT);
}
