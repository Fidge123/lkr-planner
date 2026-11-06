import {
  type CreateAssignmentInput,
  commands,
  type EmployeeWeekEvents,
  type MoveAssignmentInput,
  type MoveAssignmentResult,
  type ReorderAssignmentInput,
  type ResolvedProject,
  type UpdateAssignmentInput,
  type WeekLoadPurpose,
} from "../generated/tauri";
import { callCommand } from "./command-result";

const saveFailedMessage = "Die Änderung konnte nicht gespeichert werden.";
const moveFailedMessage = "Der Einsatz konnte nicht verschoben werden.";

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

export function createAssignment(
  input: CreateAssignmentInput,
): Promise<string> {
  return callCommand(commands.createAssignment(input), saveFailedMessage);
}

export function updateAssignment(input: UpdateAssignmentInput): Promise<null> {
  return callCommand(commands.updateAssignment(input), saveFailedMessage);
}

export function moveAssignment(
  input: MoveAssignmentInput,
): Promise<MoveAssignmentResult> {
  return callCommand(commands.moveAssignment(input), moveFailedMessage);
}

export function reorderAssignment(
  input: ReorderAssignmentInput,
): Promise<null> {
  return callCommand(commands.reorderAssignment(input), moveFailedMessage);
}

export function deleteAssignment(
  href: string,
  overrideProtection: boolean,
): Promise<null> {
  return callCommand(
    commands.deleteAssignment(href, overrideProtection),
    "Der Einsatz konnte nicht gelöscht werden.",
  );
}

/** Resolves with nothing; the failures event reports the worker days still failing. */
export function retryAppointmentSync(): Promise<void> {
  return commands.retryDayliteAppointmentSync();
}
