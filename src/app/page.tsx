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
import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import type {
  EmployeeSetting,
  PlanningContactRecord,
} from "../generated/tauri";
import type { ProjectCategoryColors } from "../services/daylite-categories";
import { AppointmentSyncAlert } from "./components/appointment-sync-alert";
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
import { useAppointmentSyncFailures } from "./hooks/use-appointment-sync-failures";
import { type HolidaysState, useHolidays } from "./hooks/use-holidays";
import type { PlanningAssignmentsState } from "./hooks/use-planning-assignments";
import { usePlanningEmployees } from "./hooks/use-planning-employees";
import { useWeekSwipe } from "./hooks/use-week-swipe";
import type { ActiveHighlight, CellEvent } from "./types";
import { activeHighlightForWeek, highlightKey, nextHighlight } from "./types";
import { getWeekDays, shiftWeekDays, toLocalISODate } from "./util";
import { swipeSettleMs } from "./week-swipe";

// The pointer, not the dragged card's box, picks the target cell and the position within the cell
const collisionDetection: CollisionDetection = (args) => {
  const pointerCollisions = pointerWithin(args);
  return pointerCollisions.length > 0
    ? pointerCollisions
    : rectIntersection(args);
};

// dnd-kit refreshes droppable rects only on per-cell resize and on a timer.
// Neither catches a cell shifting under a still pointer when an earlier row grows.
function DropzoneMeasurementTicker() {
  const { active, measureDroppableContainers } = useDndContext();
  useEffect(() => {
    if (!active) return;
    const interval = setInterval(() => measureDroppableContainers([]), 150);
    return () => clearInterval(interval);
  }, [active, measureDroppableContainers]);
  return null;
}

// A reload mid-drag shifts the cells under the pointer.
// Edge-hover navigation changes `key` mid-drag, so freezing waits for `loadedKey` or pins an empty grid.
function useFrozenDuringDrag<T>(
  value: T,
  isDragActive: boolean,
  key: string,
  loadedKey: string | null,
): T {
  const tracked = useRef({ key, frozen: value, isFrozen: false });
  const shouldFreeze = isDragActive && loadedKey === key;

  // The render that starts freezing must return its own value, not the pre-arrival one.
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
  assignmentState,
  employeeSettings = [],
  hideNonPlannableEmployees = true,
  categoryColors = {},
  onOpenIcalDialog = () => {},
  onNavigateWeek,
  onJumpToWeek,
}: Props) {
  const weekDays = getWeekDays(weekOffset, showWeekend);
  const weekStart = toLocalISODate(weekDays[0]);

  const employeeState = usePlanningEmployees();
  const holidaysState = useHolidays(weekStart);

  return (
    <PlanningGridTable
      weekDays={weekDays}
      employeeState={employeeState}
      assignmentState={assignmentState}
      employeeSettings={employeeSettings}
      hideNonPlannableEmployees={hideNonPlannableEmployees}
      holidaysState={holidaysState}
      categoryColors={categoryColors}
      onOpenIcalDialog={onOpenIcalDialog}
      onNavigateWeek={onNavigateWeek}
      onJumpToWeek={onJumpToWeek}
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
  categoryColors,
  onOpenIcalDialog,
  onNavigateWeek,
  onJumpToWeek,
}: PlanningGridTableProps) {
  const {
    employees,
    isLoading: isEmployeeLoading,
    errorMessage: employeeErrorMessage,
    reloadEmployees,
  } = employeeState;
  const {
    reloadAssignments,
    invalidateWeeksContaining,
    moveEvent,
    refreshEmployees,
    removeEvent,
  } = assignmentState;
  const {
    holidays,
    errorMessage: holidayErrorMessage,
    reloadHolidays,
  } = holidaysState;
  const appointmentSync = useAppointmentSyncFailures();
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
    moveEvent,
    refreshEmployees,
  });
  const isDragActive = drag.activePayload !== null;
  const weekStart = toLocalISODate(weekDays[0]);
  const handleAssignmentWritten = (
    employeeRef: string,
    removedUid: string | null,
  ) => {
    if (removedUid !== null) removeEvent(removedUid);
    refreshEmployees(weekStart, [employeeRef]);
  };

  const [highlight, setHighlight] = useState<ActiveHighlight | null>(null);
  const activeHighlight = activeHighlightForWeek(highlight, weekStart);
  // Deriving keeps the highlight out of other weeks; dropping it keeps navigating back from restoring it.
  if (highlight && activeHighlight === null) setHighlight(null);
  const toggleHighlight = (event: CellEvent) => {
    const key = nextHighlight(activeHighlight, highlightKey(event));
    setHighlight(key === null ? null : { key, weekStart });
  };
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

  // Gestures start only at a horizontal scroll edge, so a too-wide grid is parked on one side.
  // The incoming week must match both scroll offsets, or columns and rows jump on handover.
  const incomingRef = useRef<HTMLElement>(null);
  const hasIncoming = incomingDays !== null;
  // biome-ignore lint/correctness/useExhaustiveDependencies: hasIncoming marks the mount of the incoming week, not a value the effect body reads.
  useEffect(() => {
    if (incomingRef.current && scrollRef.current) {
      incomingRef.current.scrollLeft = scrollRef.current.scrollLeft;
      incomingRef.current.scrollTop = scrollRef.current.scrollTop;
    }
  }, [hasIncoming]);

  return (
    <section className="w-full h-full relative overflow-hidden">
      {/* Isolated so the pinned date row, which outranks the cells it covers, stays under the week sliding in. */}
      <section ref={scrollRef} className="w-full h-full overflow-auto isolate">
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
        <AppointmentSyncAlert
          failedDates={appointmentSync.failedDates}
          isRetrying={appointmentSync.isRetrying}
          onRetry={() => void appointmentSync.retry()}
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
            categoryColors={categoryColors}
            holidays={holidays}
            isEmployeeLoading={isEmployeeLoading}
            dropPreview={drag.dropPreview}
            draggedUid={drag.activePayload?.uid ?? null}
            activeHighlight={activeHighlight}
            onOpenIcalDialog={onOpenIcalDialog}
            onReloadAssignments={reloadAssignments}
            onAssignmentWritten={handleAssignmentWritten}
            onToggleHighlight={toggleHighlight}
            onJumpToWeek={onJumpToWeek}
          />
          {/* Not for Tauri: bun tests render through react-dom/server, which has no `document`. */}
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
              categoryColors={categoryColors}
              holidays={holidays}
              isEmployeeLoading={isEmployeeLoading}
              onOpenIcalDialog={onOpenIcalDialog}
              onReloadAssignments={reloadAssignments}
              onAssignmentWritten={handleAssignmentWritten}
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

// Spelled out so Tailwind's static scan finds the class names.
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
  assignmentState: PlanningGridAssignmentState;
  employeeSettings?: EmployeeSetting[];
  hideNonPlannableEmployees?: boolean;
  categoryColors?: ProjectCategoryColors;
  onOpenIcalDialog?: (employee: PlanningContactRecord) => void;
  onNavigateWeek?: (direction: -1 | 1) => void;
  onJumpToWeek?: (day: Date) => void;
}

export interface PlanningGridTableProps {
  weekDays: Date[];
  employeeState: PlanningGridEmployeesState;
  assignmentState: PlanningGridAssignmentState;
  employeeSettings: EmployeeSetting[];
  hideNonPlannableEmployees: boolean;
  holidaysState: HolidaysState;
  categoryColors: ProjectCategoryColors;
  onOpenIcalDialog: (employee: PlanningContactRecord) => void;
  onNavigateWeek?: (direction: -1 | 1) => void;
  onJumpToWeek?: (day: Date) => void;
}

export interface PlanningGridEmployeesState {
  employees: PlanningContactRecord[];
  isLoading: boolean;
  errorMessage: string | null;
  reloadEmployees: () => void;
}

export type PlanningGridAssignmentState = Omit<
  PlanningAssignmentsState,
  "lastLoadedAt" | "refreshAssignments"
>;
