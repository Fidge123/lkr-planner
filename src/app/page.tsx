import type { CollisionDetection } from "@dnd-kit/core";
import {
  DndContext,
  DragOverlay,
  PointerSensor,
  pointerWithin,
  rectIntersection,
  useDndContext,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import { useEffect, useRef } from "react";
import { createPortal } from "react-dom";
import type {
  EmployeeSetting,
  PlanningContactRecord,
} from "../generated/tauri";
import { MoveReconciliationDialog } from "./components/move-reconciliation-dialog";
import {
  AssignmentCardBody,
  assignmentCardClass,
  assignmentStripClass,
  categoryStrip,
} from "./components/timetable-cell";
import { TimetableHeader } from "./components/timetable-header";
import { TimetableRow } from "./components/timetable-row";
import { filterVisibleEmployees } from "./employee-visibility";
import type { AppointmentDragPayload } from "./hooks/use-appointment-drag";
import { useAppointmentDrag } from "./hooks/use-appointment-drag";
import { type HolidaysState, useHolidays } from "./hooks/use-holidays";
import type { PlanningAssignmentsState } from "./hooks/use-planning-assignments";
import { usePlanningEmployees } from "./hooks/use-planning-employees";
import { getWeekDays, toLocalISODate } from "./util";

// The pointer, not the dragged card's box, picks the target cell and the position within the cell
const collisionDetection: CollisionDetection = (args) => {
  const pointerCollisions = pointerWithin(args);
  return pointerCollisions.length > 0
    ? pointerCollisions
    : rectIntersection(args);
};

// dnd-kit refreshes droppable's rect from per-cell ResizeObserver (fires only when that cell resizes) and by timer (while the pointer moves).
// Neither catches a cell shifting because an earlier row grew under a still pointer, which leaves the drop landing where the cell used to be.
function DropzoneMeasurementTicker() {
  const { active, measureDroppableContainers } = useDndContext();
  useEffect(() => {
    if (!active) return;
    const interval = setInterval(() => measureDroppableContainers([]), 150);
    return () => clearInterval(interval);
  }, [active, measureDroppableContainers]);
  return null;
}

// A reload landing mid-drag would resize rows and shift every cell under the pointer, so the drop lands on whatever moved into place.
// Freezing waits for `loadedKey` to reach `key`, because edge-hover navigation changes `key` mid-drag
// and freezing before the new week arrives pins the empty grid there for the rest of the drag.
function useFrozenDuringDrag<T>(
  value: T,
  isDragActive: boolean,
  key: string,
  loadedKey: string | null,
): T {
  const tracked = useRef({ key, frozen: value, isFrozen: false });
  const shouldFreeze = isDragActive && loadedKey === key;

  // The render that starts freezing must still pass its own value through and keep it:
  // it is the render the week's data arrived on, and freezing one render earlier would pin the grid to the empty state that preceded it.
  if (
    !shouldFreeze ||
    !tracked.current.isFrozen ||
    tracked.current.key !== key
  ) {
    tracked.current = { key, frozen: value, isFrozen: shouldFreeze };
    return value;
  }

  return tracked.current.frozen;
}

export function PlanningGrid({
  weekOffset,
  showWeekend = false,
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

  const fallbackEmployeesState = usePlanningEmployees();
  const fallbackHolidaysState = useHolidays(weekStart);

  return (
    <PlanningGridTable
      weekDays={weekDays}
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
  employeeState,
  assignmentState,
  employeeSettings,
  hideNonPlannableEmployees,
  holidaysState,
  onOpenIcalDialog,
  onNavigateWeek,
}: PlanningGridTableProps) {
  const {
    employees,
    isLoading: isEmployeeLoading,
    errorMessage: employeeErrorMessage,
    reloadEmployees,
  } = employeeState;
  const { reloadAssignments, invalidateWeeksContaining } = assignmentState;
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
    invalidateWeeksContaining,
  });
  const isDragActive = drag.activePayload !== null;
  const weekStart = toLocalISODate(weekDays[0]);
  const {
    eventsByEmployee,
    errorsByEmployee,
    errorMessage: assignmentErrorMessage,
  } = useFrozenDuringDrag(
    assignmentState,
    isDragActive,
    weekStart,
    assignmentState.loadedWeekStart,
  );

  // WKWebView can leave a card's previous pixels when a reload rewrites the cells, so a re-slotted card renders torn until a repaint clears it.
  // Promoting the grid to its own compositing layer and releasing it on the next frame re-rasterizes every cell without affecting layout.
  const gridRef = useRef<HTMLTableElement>(null);
  // biome-ignore lint/correctness/useExhaustiveDependencies: eventsByEmployee is the repaint trigger, not a value the effect body reads.
  useEffect(() => {
    const grid = gridRef.current;
    if (!grid) return;
    grid.style.transform = "translateZ(0)";
    const frame = requestAnimationFrame(() => {
      grid.style.transform = "";
    });
    return () => cancelAnimationFrame(frame);
  }, [eventsByEmployee]);

  return (
    <section className="w-full h-full overflow-auto">
      <ReloadableAlert
        message={employeeErrorMessage}
        onReload={reloadEmployees}
      />
      <ReloadableAlert
        message={assignmentErrorMessage}
        onReload={reloadAssignments}
      />
      <ReloadableAlert
        message={holidayErrorMessage}
        onReload={reloadHolidays}
        variant="warning"
      />
      {drag.errorMessage ? (
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
        collisionDetection={collisionDetection}
        onDragStart={drag.onDragStart}
        onDragMove={drag.onDragMove}
        onDragEnd={drag.onDragEnd}
        onDragCancel={drag.onDragCancel}
      >
        <DropzoneMeasurementTicker />
        <table ref={gridRef} className="table table-fixed border-collapse">
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
                key={employee.self || `employee-${index}`}
                employee={employee}
                calendarEvents={eventsByEmployee[employee.self] ?? []}
                calendarError={errorsByEmployee[employee.self] ?? null}
                week={{ days: weekDays, holidayDates }}
                employeeSetting={
                  employeeSettings.find(
                    (s) => s.dayliteContactReference === employee.self,
                  ) ?? null
                }
                dropPreview={drag.dropPreview}
                draggedUid={drag.activePayload?.uid ?? null}
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
              <DragOverlay style={{ pointerEvents: "none" }}>
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
    </section>
  );
}

// Spelled out rather than interpolated so the class names survive Tailwind's static scan of the source.
const alertVariantClass = {
  error: "alert-error",
  warning: "alert-warning",
} as const;

function ReloadableAlert({
  message,
  onReload,
  variant = "error",
}: ReloadableAlertProps) {
  if (!message) return null;
  return (
    <section className={`alert ${alertVariantClass[variant]} m-4`}>
      <span>{message}</span>
      <button type="button" className="btn btn-sm" onClick={onReload}>
        Erneut laden
      </button>
    </section>
  );
}

interface ReloadableAlertProps {
  message: string | null;
  onReload: () => void;
  variant?: keyof typeof alertVariantClass;
}

function DragPreviewCard({ payload }: { payload: AppointmentDragPayload }) {
  return (
    <span
      className={`${assignmentCardClass} ${assignmentStripClass} bg-base-200 text-base-content shadow-lg`}
      style={categoryStrip(payload.categoryColor)}
    >
      <AssignmentCardBody
        startTime={null}
        endTime={null}
        title={payload.title}
      />
    </span>
  );
}

interface Props {
  weekOffset: number;
  showWeekend?: boolean;
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
  employeeState: PlanningGridEmployeesState;
  assignmentState: PlanningGridAssignmentState;
  employeeSettings: EmployeeSetting[];
  hideNonPlannableEmployees: boolean;
  holidaysState: HolidaysState;
  onOpenIcalDialog: (employee: PlanningContactRecord) => void;
  onNavigateWeek?: (direction: -1 | 1) => void;
}

export interface PlanningGridEmployeesState {
  employees: PlanningContactRecord[];
  isLoading: boolean;
  errorMessage: string | null;
  reloadEmployees: () => void;
}

export type PlanningGridAssignmentState = PlanningAssignmentsState;
