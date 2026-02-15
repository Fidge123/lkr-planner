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
      {isLoading && projects.length === 0 ? (
        <p className="p-4 text-base-content">Projekte werden geladen...</p>
      ) : null}
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
