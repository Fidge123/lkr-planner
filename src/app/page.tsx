import type {
  EmployeeSetting,
  PlanningContactRecord,
  PlanningProjectRecord,
} from "../generated/tauri";
import { ProjectTable } from "./components/project-table";
import { TimetableHeader } from "./components/timetable-header";
import { TimetableRow } from "./components/timetable-row";
import { filterVisibleEmployees } from "./employee-visibility";
import { type HolidaysState, useHolidays } from "./hooks/use-holidays";
import type { PlanningAssignmentsState } from "./hooks/use-planning-assignments";
import { usePlanningEmployees } from "./hooks/use-planning-employees";
import { usePlanningProjects } from "./hooks/use-planning-projects";
import { getWeekDays, toLocalISODate } from "./util";

// The optional state props are a test seam: production callers pass only
// weekOffset and assignmentState, tests inject prepared states.
export function PlanningGrid({
  weekOffset,
  showWeekend = false,
  projectState,
  employeeState,
  assignmentState,
  employeeSettings = [],
  hideNonPlannableEmployees = true,
  holidaysState,
  onOpenIcalDialog = () => {},
}: Props) {
  const weekDays = getWeekDays(weekOffset, showWeekend);
  const weekStart = toLocalISODate(weekDays[0]);

  const fallbackProjectsState = usePlanningProjects();
  const fallbackEmployeesState = usePlanningEmployees();
  const fallbackHolidaysState = useHolidays(weekStart);

  const { projects, isLoading, errorMessage, reloadProjects } =
    projectState ?? fallbackProjectsState;
  const {
    employees,
    isLoading: isEmployeeLoading,
    errorMessage: employeeErrorMessage,
    reloadEmployees,
  } = employeeState ?? fallbackEmployeesState;
  const {
    eventsByEmployee,
    errorsByEmployee,
    isLoading: isAssignmentsLoading,
    errorMessage: assignmentErrorMessage,
    reloadAssignments,
  } = assignmentState;
  const {
    holidays,
    errorMessage: holidayErrorMessage,
    reloadHolidays,
  } = holidaysState ?? fallbackHolidaysState;
  const holidayByDate = new Map(holidays.map((h) => [h.date, h.name]));
  const holidayDates = new Set(holidays.map((h) => h.date));
  const visibleEmployees = filterVisibleEmployees(
    employees,
    employeeSettings,
    hideNonPlannableEmployees,
  );

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
          <button type="button" className="btn btn-sm" onClick={reloadHolidays}>
            Erneut laden
          </button>
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
              const isoDay = toLocalISODate(day);
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
          {visibleEmployees.map((employee, index) => (
            <TimetableRow
              key={buildEmployeeRowKey(employee, index)}
              employee={employee}
              calendarEvents={eventsByEmployee[employee.self] ?? []}
              calendarError={errorsByEmployee[employee.self] ?? null}
              week={{ days: weekDays, holidayDates }}
              employeeSetting={
                employeeSettings.find(
                  (s) => s.dayliteContactReference === employee.self,
                ) ?? null
              }
              onOpenIcalDialog={onOpenIcalDialog}
              onReloadAssignments={reloadAssignments}
            />
          ))}
          {!isEmployeeLoading && visibleEmployees.length === 0 ? (
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

      <ProjectTable projects={projects} isLoading={isLoading} />
    </section>
  );
}

interface Props {
  weekOffset: number;
  showWeekend?: boolean;
  projectState?: PlanningGridProjectsState;
  employeeState?: PlanningGridEmployeesState;
  assignmentState: PlanningGridAssignmentState;
  employeeSettings?: EmployeeSetting[];
  hideNonPlannableEmployees?: boolean;
  holidaysState?: HolidaysState;
  onOpenIcalDialog?: (employee: PlanningContactRecord) => void;
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
