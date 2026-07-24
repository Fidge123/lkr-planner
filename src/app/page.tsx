import {
  DndContext,
  DragOverlay,
  PointerSensor,
  useDndContext,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import { useEffect, useRef, useState } from "react";
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

/**
 * dnd-kit only refreshes a droppable's measured rect via a per-cell ResizeObserver
 * (fires only when that cell's own box resizes) or a periodic timer that only
 * re-schedules while the pointer is moving. Neither catches a cell shifting
 * position because an earlier row grew — e.g. another employee's assignment data
 * finishing its load mid-drag — while the pointer holds still over the target,
 * which desyncs the drop target from the cursor or makes the drop miss entirely.
 * Rendered inside DndContext; forces a plain, movement-independent re-measure for
 * as long as a drag is active.
 */
function DropzoneMeasurementTicker() {
  const { active, measureDroppableContainers } = useDndContext();
  useEffect(() => {
    if (!active) return;
    const interval = setInterval(() => measureDroppableContainers([]), 150);
    return () => clearInterval(interval);
  }, [active, measureDroppableContainers]);
  return null;
}

/** How long (ms) a week key must be stable before useFrozenDuringDrag starts freezing against it. */
const freezeArmDelayMs = 350;

/**
 * Holds `value` steady for as long as `isDragActive` is true, resuming live
 * updates the moment it goes false. A background reload landing mid-drag for
 * the SAME week (e.g. from rescheduling a different employee moments earlier)
 * would otherwise grow or shrink rows and shift every cell under the pointer,
 * so the drop can land on whatever now happens to be there instead of what the
 * user was aiming at.
 *
 * `key` (the viewed week's start date) resets protection on every navigation:
 * `weekDays` (hence `key`) updates synchronously with the navigation, but
 * `assignmentState` only catches up on a later render via a separate effect,
 * so trusting the very first post-navigation value would freeze on a stale
 * snapshot of the PREVIOUS week and never pick up the new week's real data.
 * Freezing only re-arms `freezeArmDelayMs` after `key` last changed, by which
 * point the new week's fetch (cached or not) has had time to land.
 */
function useFrozenDuringDrag<T>(
  value: T,
  isDragActive: boolean,
  key: string,
): T {
  const [tracked, setTracked] = useState({ key, armed: false, frozen: value });
  const armTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (armTimer.current !== null) {
      clearTimeout(armTimer.current);
    }
    armTimer.current = setTimeout(() => {
      setTracked((current) =>
        current.key === key ? { ...current, armed: true } : current,
      );
    }, freezeArmDelayMs);
    return () => {
      if (armTimer.current !== null) {
        clearTimeout(armTimer.current);
      }
    };
  }, [key]);

  if (tracked.key !== key) {
    setTracked({ key, armed: false, frozen: value });
    return value;
  }

  if (!tracked.armed || !isDragActive) {
    if (tracked.frozen !== value) {
      setTracked((current) => ({ ...current, frozen: value }));
    }
    return value;
  }

  return tracked.frozen;
}

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
  const { reloadAssignments } = assignmentState;
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
  const isDragActive = drag.activePayload !== null;
  // A background reload for the SAME week (e.g. from a drop moments earlier)
  // must not resize any row while this drag is active - see useFrozenDuringDrag.
  // Keyed on the viewed week so edge-hover navigation mid-drag still shows the
  // newly-navigated week's data immediately instead of the old week's.
  const {
    eventsByEmployee,
    errorsByEmployee,
    isLoading: isAssignmentsLoading,
    errorMessage: assignmentErrorMessage,
  } = useFrozenDuringDrag(
    assignmentState,
    isDragActive,
    toLocalISODate(weekDays[0]),
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
        <DropzoneMeasurementTicker />
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
