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
      // Minimal records carry only reference and name; the category drives the
      // color strip on each result.
      fullRecords: true,
      start: null,
      sort: "name",
    }),
    "Projekte konnten nicht geladen werden.",
  );

  return (result.results ?? []).slice(0, DISPLAY_LIMIT);
}
