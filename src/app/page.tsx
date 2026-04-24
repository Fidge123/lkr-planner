import type {
  EmployeeSetting,
  PlanningContactRecord,
  PlanningProjectRecord,
} from "../generated/tauri";
import { TimetableHeader } from "./components/timetable-header";
import { TimetableRow } from "./components/timetable-row";
import type { PlanningAssignmentsState } from "./use-planning-assignments";
import { usePlanningAssignments } from "./use-planning-assignments";
import { usePlanningEmployees } from "./use-planning-employees";
import { usePlanningProjects } from "./use-planning-projects";
import { type HolidaysState, useHolidays } from "./use-holidays";
import { getWeekDays } from "./util";

export function PlanningGrid({
  weekOffset,
  projectState,
  employeeState,
  assignmentState,
  employeeSettings,
  onOpenIcalDialog,
}: Props) {
  const weekDays = getWeekDays(weekOffset);
  const weekStart = weekDays[0].toISOString().slice(0, 10);

  const planningProjectsState = usePlanningProjects();
  const planningEmployeesState = usePlanningEmployees();
  const planningAssignmentsState = usePlanningAssignments(weekStart);
  const holidaysState = useHolidays(weekStart);

  const resolvedProjectState = projectState ?? planningProjectsState;
  const resolvedEmployeeState = employeeState ?? planningEmployeesState;
  const resolvedAssignmentState = assignmentState ?? planningAssignmentsState;

  return (
    <PlanningGridTable
      weekOffset={weekOffset}
      projectState={resolvedProjectState}
      employeeState={resolvedEmployeeState}
      assignmentState={resolvedAssignmentState}
      employeeSettings={employeeSettings ?? []}
      holidaysState={holidaysState}
      onOpenIcalDialog={onOpenIcalDialog ?? (() => {})}
    />
  );
}

export function PlanningGridTable({
  weekOffset,
  projectState,
  employeeState,
  assignmentState,
  employeeSettings,
  holidaysState,
  onOpenIcalDialog,
}: PlanningGridTableProps) {
  const weekDays = getWeekDays(weekOffset);
  const { projects, isLoading, errorMessage, reloadProjects } = projectState;
  const {
    employees,
    isLoading: isEmployeeLoading,
    errorMessage: employeeErrorMessage,
    reloadEmployees,
  } = employeeState;
  const {
    eventsByEmployee,
    errorsByEmployee,
    isLoading: isAssignmentsLoading,
    errorMessage: assignmentErrorMessage,
    reloadAssignments,
  } = assignmentState;
  const { holidays, errorMessage: holidayErrorMessage } = holidaysState ?? {
    holidays: [],
    errorMessage: null,
  };
  const holidayByDate = new Map(holidays.map((h) => [h.date, h.name]));
  const holidayDates = new Set(holidays.map((h) => h.date));

  return (
    <section className="w-full h-full overflow-auto">
      {errorMessage ? (
        <section className="alert alert-error m-4">
          <span>{errorMessage}</span>
          <button type="button" className="btn btn-sm" onClick={reloadProjects}>
            Erneut laden
          </button>
        </section>
      ) : null}
      {employeeErrorMessage ? (
        <section className="alert alert-error m-4">
          <span>{employeeErrorMessage}</span>
          <button
            type="button"
            className="btn btn-sm"
            onClick={reloadEmployees}
          >
            Erneut laden
          </button>
        </section>
      ) : null}
      {assignmentErrorMessage ? (
        <section className="alert alert-error m-4">
          <span>{assignmentErrorMessage}</span>
          <button
            type="button"
            className="btn btn-sm"
            onClick={reloadAssignments}
          >
            Erneut laden
          </button>
        </section>
      ) : null}
      {holidayErrorMessage ? (
        <section className="alert alert-warning m-4">
          <span>{holidayErrorMessage}</span>
        </section>
      ) : null}
      {isAssignmentsLoading ? (
        <p className="px-4 py-2 text-base-content/70">
          Einsätze werden geladen...
        </p>
      ) : null}
      <table className="table table-fixed border-collapse">
        <thead className="text-base-content">
          <tr>
            <th className="w-40 p-4 font-bold">Mitarbeiter</th>
            {weekDays.map((day) => {
              const isoDay = day.toISOString().slice(0, 10);
              return (
                <TimetableHeader
                  key={day.getTime()}
                  day={day}
                  holiday={holidayByDate.get(isoDay)}
                />
              );
            })}
          </tr>
        </thead>
        <tbody>
          {employees.map((employee, index) => (
            <TimetableRow
              key={buildEmployeeRowKey(employee, index)}
              employee={employee}
              calendarEvents={eventsByEmployee[employee.self] ?? []}
              calendarError={errorsByEmployee[employee.self] ?? null}
              weekDays={weekDays}
              employeeSetting={
                employeeSettings.find(
                  (s) => s.dayliteContactReference === employee.self,
                ) ?? null
              }
              holidayDates={holidayDates}
              onRetry={reloadAssignments}
              onOpenIcalDialog={onOpenIcalDialog}
            />
          ))}
          {!isEmployeeLoading && employees.length === 0 ? (
            <tr key="no-employees-row">
              <td
                className="p-4 text-base-content/70"
                colSpan={weekDays.length + 1}
              >
                Keine Mitarbeiter gefunden
              </td>
            </tr>
          ) : null}
        </tbody>
      </table>

      <section className="p-4 border-t border-base-300">
        <h2 className="text-lg font-semibold">Geladene Projekte</h2>
        {isLoading ? (
          <p className="mt-2 text-base-content/70">
            Projekte werden geladen...
          </p>
        ) : null}
        {!isLoading && projects.length === 0 ? (
          <p className="mt-2 text-base-content/70">Keine Projekte gefunden</p>
        ) : null}
        {!isLoading && projects.length > 0 ? (
          <table className="table table-sm mt-3">
            <thead>
              <tr>
                <th>Projekt</th>
                <th>Status</th>
                <th>Fällig</th>
              </tr>
            </thead>
            <tbody>
              {projects.map((project, index) => (
                <tr key={buildProjectRowKey(project, index)}>
                  <td>{project.name}</td>
                  <td>{toGermanProjectStatus(project.status)}</td>
                  <td>{formatGermanDate(project.due)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        ) : null}
      </section>
    </section>
  );
}

interface Props {
  weekOffset: number;
  projectState?: PlanningGridProjectsState;
  employeeState?: PlanningGridEmployeesState;
  assignmentState?: PlanningGridAssignmentState;
  employeeSettings?: EmployeeSetting[];
  onOpenIcalDialog?: (employee: PlanningContactRecord) => void;
}

interface PlanningGridTableProps {
  weekOffset: number;
  projectState: PlanningGridProjectsState;
  employeeState: PlanningGridEmployeesState;
  assignmentState: PlanningGridAssignmentState;
  employeeSettings: EmployeeSetting[];
  holidaysState?: HolidaysState;
  onOpenIcalDialog: (employee: PlanningContactRecord) => void;
}

export interface PlanningGridProjectsState {
  projects: PlanningProjectRecord[];
  isLoading: boolean;
  errorMessage: string | null;
  reloadProjects: () => void;
}

export interface PlanningGridEmployeesState {
  employees: PlanningContactRecord[];
  isLoading: boolean;
  errorMessage: string | null;
  reloadEmployees: () => void;
}

export type PlanningGridAssignmentState = PlanningAssignmentsState;

function toGermanProjectStatus(
  status: PlanningProjectRecord["status"],
): string {
  if (status === "new_status") {
    return "Neu";
  }
  if (status === "in_progress") {
    return "In Arbeit";
  }
  if (status === "done") {
    return "Erledigt";
  }
  if (status === "abandoned") {
    return "Abgebrochen";
  }
  if (status === "cancelled") {
    return "Storniert";
  }
  if (status === "deferred") {
    return "Zurückgestellt";
  }

  return "Unbekannt";
}

function formatGermanDate(isoDate: string | null | undefined): string {
  if (!isoDate) {
    return "Kein Termin";
  }

  const date = new Date(isoDate);
  if (Number.isNaN(date.getTime())) {
    return "Kein Termin";
  }

  return date.toLocaleDateString("de-DE", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  });
}

function buildEmployeeRowKey(
  employee: PlanningContactRecord,
  index: number,
): string {
  const stableReference = employee.self.trim();
  if (stableReference.length > 0) {
    return stableReference;
  }

  const stableName = (employee.nickname ?? employee.full_name ?? "").trim();
  if (stableName.length > 0) {
    return `employee-${stableName}-${index}`;
  }

  return `employee-empty-${index}`;
}

function buildProjectRowKey(
  project: PlanningProjectRecord,
  index: number,
): string {
  const stableReference =
    typeof project.self === "string" ? project.self.trim() : "";
  if (stableReference.length > 0) {
    return stableReference;
  }

  const stableName = project.name.trim();
  if (stableName.length > 0) {
    return `project-${stableName}-${index}`;
  }

  return `project-empty-${index}`;
}
