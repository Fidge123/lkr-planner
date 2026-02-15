import { employees } from "../data/dummy-data";
import type { DayliteProjectRecord } from "../domain/planning";
import { TimetableHeader } from "./components/timetable-header";
import { TimetableRow } from "./components/timetable-row";
import { usePlanningProjects } from "./use-planning-projects";
import { getWeekDays } from "./util";

export function PlanningGrid({ weekOffset, projectState }: Props) {
  const planningProjectsState = usePlanningProjects();
  const resolvedProjectState = projectState ?? planningProjectsState;

  return (
    <PlanningGridTable
      weekOffset={weekOffset}
      projectState={resolvedProjectState}
    />
  );
}

export function PlanningGridTable({
  weekOffset,
  projectState,
}: PlanningGridTableProps) {
  const weekDays = getWeekDays(weekOffset);
  const { projects, isLoading, errorMessage, reloadProjects } = projectState;

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
      <table className="table table-zebra table-fixed border-collapse">
        <thead className="text-base-content">
          <tr className="divide-x divide-slate-300 bg-base-200">
            <th className="w-36 p-4 font-bold">Mitarbeiter</th>
            {weekDays.map((day) => (
              <TimetableHeader key={day.getTime()} day={day} />
            ))}
          </tr>
        </thead>
        <tbody>
          {employees.map((employee) => (
            <TimetableRow
              key={employee.self}
              employee={employee}
              projects={projects}
              weekDays={weekDays}
            />
          ))}
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
          <p className="mt-2 text-base-content/70">
            Keine Projekte im Standard-Filter gefunden
          </p>
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
              {projects.map((project) => (
                <tr key={project.self}>
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
}

interface PlanningGridTableProps {
  weekOffset: number;
  projectState: PlanningGridProjectsState;
}

export interface PlanningGridProjectsState {
  projects: DayliteProjectRecord[];
  isLoading: boolean;
  errorMessage: string | null;
  reloadProjects: () => void;
}

function toGermanProjectStatus(status: DayliteProjectRecord["status"]): string {
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

function formatGermanDate(isoDate: string | undefined): string {
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
