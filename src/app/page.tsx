import {
  DndContext,
  DragOverlay,
  PointerSensor,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import { createPortal } from "react-dom";
import type {
  EmployeeSetting,
  PlanningContactRecord,
  PlanningProjectRecord,
} from "../generated/tauri";
import { MoveReconciliationDialog } from "./components/move-reconciliation-dialog";
import { ProjectTable } from "./components/project-table";
import { TimetableHeader } from "./components/timetable-header";
import { TimetableRow } from "./components/timetable-row";
import { filterVisibleEmployees } from "./employee-visibility";
import type { AppointmentDragPayload } from "./hooks/use-appointment-drag";
import { useAppointmentDrag } from "./hooks/use-appointment-drag";
import { type HolidaysState, useHolidays } from "./hooks/use-holidays";
import type { PlanningAssignmentsState } from "./hooks/use-planning-assignments";
import { usePlanningEmployees } from "./hooks/use-planning-employees";
import { usePlanningProjects } from "./hooks/use-planning-projects";
import { getWeekDays, toLocalISODate } from "./util";

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
  onNavigateWeek,
}: Props) {
  const weekDays = getWeekDays(weekOffset, showWeekend);
  const weekStart = toLocalISODate(weekDays[0]);

  const fallbackProjectsState = usePlanningProjects();
  const fallbackEmployeesState = usePlanningEmployees();
  const fallbackHolidaysState = useHolidays(weekStart);

  return (
    <PlanningGridTable
      weekDays={weekDays}
      projectState={projectState ?? fallbackProjectsState}
      employeeState={employeeState ?? fallbackEmployeesState}
      assignmentState={assignmentState}
      employeeSettings={employeeSettings}
      hideNonPlannableEmployees={hideNonPlannableEmployees}
      holidaysState={holidaysState ?? fallbackHolidaysState}
      onOpenIcalDialog={onOpenIcalDialog}
      onNavigateWeek={onNavigateWeek}
    />
  );
}

export function PlanningGridTable({
  weekDays,
  projectState,
  employeeState,
  assignmentState,
  employeeSettings,
  hideNonPlannableEmployees,
  holidaysState,
  onOpenIcalDialog,
  onNavigateWeek,
}: PlanningGridTableProps) {
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
  const {
    holidays,
    errorMessage: holidayErrorMessage,
    reloadHolidays,
  } = holidaysState;
  const holidayByDate = new Map(holidays.map((h) => [h.date, h.name]));
  const holidayDates = new Set(holidays.map((h) => h.date));
  const visibleEmployees = filterVisibleEmployees(
    employees,
    employeeSettings,
    hideNonPlannableEmployees,
  );

  // A small pointer distance keeps plain clicks opening the edit modal.
  const dragSensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } }),
  );
  const drag = useAppointmentDrag({
    onNavigateWeek: onNavigateWeek ?? (() => {}),
    reloadAssignments,
  });

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
      {drag.errorMessage ? (
        // Fixed-position toast so the message is visible no matter where the
        // grid is scrolled when the drop fails.
        <section className="toast toast-top toast-center z-50">
          <section className="alert alert-error">
            <span>{drag.errorMessage}</span>
            <button
              type="button"
              className="btn btn-sm"
              onClick={drag.clearError}
            >
              Schließen
            </button>
          </section>
        </section>
      ) : null}
      <DndContext
        sensors={dragSensors}
        onDragStart={drag.onDragStart}
        onDragMove={drag.onDragMove}
        onDragEnd={drag.onDragEnd}
        onDragCancel={drag.onDragCancel}
      >
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
        {/* The document guard is not about Tauri: bun tests render this grid
            through react-dom/server, where portals and `document` do not exist. */}
        {typeof document === "undefined"
          ? null
          : createPortal(
              <DragOverlay>
                {drag.activePayload ? (
                  <DragPreviewCard payload={drag.activePayload} />
                ) : null}
              </DragOverlay>,
              document.body,
            )}
      </DndContext>

      <MoveReconciliationDialog
        reconciliation={drag.reconciliation}
        onResolved={drag.resolveReconciliation}
      />

      <ProjectTable projects={projects} isLoading={isLoading} />
    </section>
  );
}

/** Pointer-following preview of the dragged card, portal-mounted so it survives week navigation. */
function DragPreviewCard({ payload }: { payload: AppointmentDragPayload }) {
  return (
    <span
      className={`flex items-center w-full gap-4 p-2 rounded-lg text-base-100 shadow-lg ${payload.color}`}
    >
      <h4 className="flex-1 min-w-0 font-medium">{payload.title}</h4>
    </span>
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
  onNavigateWeek?: (direction: -1 | 1) => void;
}

export interface PlanningGridTableProps {
  weekDays: Date[];
  projectState: PlanningGridProjectsState;
  employeeState: PlanningGridEmployeesState;
  assignmentState: PlanningGridAssignmentState;
  employeeSettings: EmployeeSetting[];
  hideNonPlannableEmployees: boolean;
  holidaysState: HolidaysState;
  onOpenIcalDialog: (employee: PlanningContactRecord) => void;
  onNavigateWeek?: (direction: -1 | 1) => void;
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
