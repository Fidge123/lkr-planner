import { commands, type DayliteProjectSummary } from "../generated/tauri";
import { readDayliteApiErrorMessage } from "./daylite-service-helpers";

// Up to 5 active projects (new_status / in_progress) matching the filter term,
// sorted by name. The backend enforces the limit, status filter and name sort;
// this function just shapes the request and unwraps the result.
export async function searchProjectsForAssignmentPicker(
  searchTerm: string,
): Promise<DayliteProjectSummary[]> {
  const result = await commands.dayliteSearchProjects({
    searchTerm,
    limit: 5,
    statuses: ["new_status", "in_progress"],
    fullRecords: null,
    start: null,
    sort: "name",
  });

  if (result.status === "error") {
    throw new Error(
      readDayliteApiErrorMessage(
        result.error,
        "Projekte konnten nicht geladen werden.",
      ),
    );
  }

  return result.data.results;
}
