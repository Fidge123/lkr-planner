import { commands, type DayliteProjectSummary } from "../generated/tauri";
import { readDayliteApiErrorMessage } from "./daylite-service-helpers";

let cache: DayliteProjectSummary[] | null = null;

export async function loadProjectsForAssignmentPicker(): Promise<
  DayliteProjectSummary[]
> {
  if (cache) return cache;

  const result = await commands.dayliteSearchProjects({
    searchTerm: "",
    limit: null,
    statuses: ["new_status", "in_progress"],
  });

  if (result.status === "error") {
    throw new Error(
      readDayliteApiErrorMessage(
        result.error,
        "Projekte konnten nicht geladen werden.",
      ),
    );
  }

  cache = result.data.results;
  return cache;
}

export function test_resetAssignmentProjectPickerCache(): void {
  cache = null;
}
