import { commands, type DayliteProjectSummary } from "../generated/tauri";
import { unwrapCommandResult } from "./command-result";

const DISPLAY_LIMIT = 5;
// Candidate pool fetched from Daylite before the name sort. Daylite applies its
// own ordering when truncating to `limit`, so fetching only 5 would let it drop
// matches that sort earlier by name. Fetching a wider pool lets the backend
// name-sort pick the true alphabetically-first matches; we then show the top 5.
const CANDIDATE_LIMIT = 50;

// Up to 5 active projects (new_status / in_progress) matching the filter term,
// sorted by name. The backend applies the status filter and name sort over the
// candidate pool; this function shapes the request and trims to the display
// limit.
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

  // Daylite omits `results` when a search has no matches; the Rust side
  // defaults it to an empty list, but that makes the generated binding
  // optional too.
  return (result.results ?? []).slice(0, DISPLAY_LIMIT);
}
