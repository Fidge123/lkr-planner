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
import { WeekTable } from "./components/week-table";
import { filterVisibleEmployees } from "./employee-visibility";
import type { AppointmentDragPayload } from "./hooks/use-appointment-drag";
import { useAppointmentDrag } from "./hooks/use-appointment-drag";
import { type HolidaysState, useHolidays } from "./hooks/use-holidays";
import type { PlanningAssignmentsState } from "./hooks/use-planning-assignments";
import { usePlanningEmployees } from "./hooks/use-planning-employees";
import { useWeekSwipe } from "./hooks/use-week-swipe";
import { getWeekDays, shiftWeekDays, toLocalISODate } from "./util";
import { swipeSettleMs } from "./week-swipe";

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

  const scrollRef = useRef<HTMLElement>(null);
  const swipe = useWeekSwipe({
    containerRef: scrollRef,
    onNavigate: onNavigateWeek ?? (() => {}),
    isDisabled: !onNavigateWeek || isDragActive,
  });
  const incomingDays = swipe ? shiftWeekDays(weekDays, swipe.direction) : null;
  const incomingWeek = incomingDays
    ? assignmentState.getCachedWeek(toLocalISODate(incomingDays[0]))
    : null;

  // A gesture can only start at a scroll edge, so a grid too wide for the window is parked
  // to one side; the incoming week has to arrive at the same offset or the columns jump on handover.
  const incomingRef = useRef<HTMLElement>(null);
  const hasIncoming = incomingDays !== null;
  // biome-ignore lint/correctness/useExhaustiveDependencies: hasIncoming marks the mount of the incoming week, not a value the effect body reads.
  useEffect(() => {
    if (incomingRef.current && scrollRef.current) {
      incomingRef.current.scrollLeft = scrollRef.current.scrollLeft;
    }
  }, [hasIncoming]);

  return (
    <section className="w-full h-full relative overflow-hidden">
      <section ref={scrollRef} className="w-full h-full overflow-auto">
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
          <WeekTable
            weekDays={weekDays}
            employees={visibleEmployees}
            employeeSettings={employeeSettings}
            eventsByEmployee={eventsByEmployee}
            errorsByEmployee={errorsByEmployee}
            holidays={holidays}
            isEmployeeLoading={isEmployeeLoading}
            dropPreview={drag.dropPreview}
            draggedUid={drag.activePayload?.uid ?? null}
            onOpenIcalDialog={onOpenIcalDialog}
            onReloadAssignments={reloadAssignments}
          />
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
      </section>

      {swipe && incomingDays ? (
        <aside
          ref={incomingRef}
          inert
          // Without pointer-events-none the covering week swallows the wheel events that drive the gesture.
          className="absolute inset-0 overflow-hidden bg-base-100 shadow-2xl pointer-events-none"
          style={{
            transform: `translateX(${swipe.translatePercent}%)`,
            transition: swipe.isAnimating
              ? `transform ${swipeSettleMs}ms ease-out`
              : "none",
          }}
        >
          {/* Its own context keeps the incoming week's cells out of the running drag's drop targets. */}
          <DndContext>
            <WeekTable
              weekDays={incomingDays}
              employees={visibleEmployees}
              employeeSettings={employeeSettings}
              eventsByEmployee={incomingWeek?.eventsByEmployee ?? {}}
              errorsByEmployee={incomingWeek?.errorsByEmployee ?? {}}
              holidays={holidays}
              isEmployeeLoading={isEmployeeLoading}
              onOpenIcalDialog={onOpenIcalDialog}
              onReloadAssignments={reloadAssignments}
            />
          </DndContext>
        </aside>
      ) : null}

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
