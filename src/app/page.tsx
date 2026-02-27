import type { DayliteContactRecord } from "../domain/planning";
import type { PlanningProjectRecord } from "../generated/tauri";
import { TimetableHeader } from "./components/timetable-header";
import { TimetableRow } from "./components/timetable-row";
import { usePlanningEmployees } from "./use-planning-employees";
import { usePlanningProjects } from "./use-planning-projects";
import { getWeekDays } from "./util";

export function PlanningGrid({
  weekOffset,
  projectState,
  employeeState,
}: Props) {
  const planningProjectsState = usePlanningProjects();
  const planningEmployeesState = usePlanningEmployees();
  const resolvedProjectState = projectState ?? planningProjectsState;
  const resolvedEmployeeState = employeeState ?? planningEmployeesState;

  return (
    <PlanningGridTable
      weekOffset={weekOffset}
      projectState={resolvedProjectState}
      employeeState={resolvedEmployeeState}
    />
  );
}

export function PlanningGridTable({
  weekOffset,
  projectState,
  employeeState,
}: PlanningGridTableProps) {
  const weekDays = getWeekDays(weekOffset);
  const { projects, isLoading, errorMessage, reloadProjects } = projectState;
  const {
    employees,
    isLoading: isEmployeeLoading,
    errorMessage: employeeErrorMessage,
    reloadEmployees,
  } = employeeState;

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
      <table className="table table-fixed border-collapse">
        <thead className="text-base-content">
          <tr>
            <th className="w-36 p-4 font-bold">Mitarbeiter</th>
            {weekDays.map((day) => (
              <TimetableHeader key={day.getTime()} day={day} />
            ))}
          </tr>
        </thead>
        <tbody>
          {employees.map((employee, index) => (
            <TimetableRow
              key={buildEmployeeRowKey(employee, index)}
              employee={employee}
              projects={projects}
              weekDays={weekDays}
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
}

interface PlanningGridTableProps {
  weekOffset: number;
  projectState: PlanningGridProjectsState;
  employeeState: PlanningGridEmployeesState;
}

export interface PlanningGridProjectsState {
  projects: PlanningProjectRecord[];
  isLoading: boolean;
  errorMessage: string | null;
  reloadProjects: () => void;
}

export interface PlanningGridEmployeesState {
  employees: DayliteContactRecord[];
  isLoading: boolean;
  errorMessage: string | null;
  reloadEmployees: () => void;
}

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
  employee: DayliteContactRecord,
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
