import {
  commands,
  type EmployeeWeekEvents,
  type ResolvedProject,
  type WeekLoadPurpose,
} from "../generated/tauri";
import { callCommand } from "./command-result";

/** Without employee references, every employee with a calendar is loaded. */
export function loadWeekEvents(
  weekStart: string,
  employeeReferences: string[] | null,
  purpose: WeekLoadPurpose,
): Promise<EmployeeWeekEvents[]> {
  return callCommand(
    commands.loadWeekEvents(weekStart, employeeReferences, purpose),
    "Die Einsätze konnten nicht geladen werden.",
  );
}

/** A reference that neither Daylite nor the cache could resolve maps to null. */
export function resolveWeekProjects(
  projectRefs: string[],
  forceRefresh: boolean,
): Promise<Partial<Record<string, ResolvedProject | null>>> {
  return callCommand(
    commands.resolveWeekProjects(projectRefs, forceRefresh),
    "Die Projekte konnten nicht geladen werden.",
  );
}
