import type {
  DragEndEvent,
  DragMoveEvent,
  DragStartEvent,
} from "@dnd-kit/core";
import { useCallback, useEffect, useRef, useState } from "react";
import { commands, type MoveAssignmentResult } from "../../generated/tauri";
import type { MoveReconciliation } from "../components/move-reconciliation-dialog";

/** Identity and display data of the assignment card being dragged. */
export interface AppointmentDragPayload {
  uid: string;
  href: string;
  projectRef: string;
  employeeRef: string;
  date: string;
  /** Project name; doubles as the persisted event summary on drop. */
  title: string;
  color: string;
}

/** The day cell a card is dropped on. */
export interface DropCellTarget {
  employeeRef: string;
  date: string;
}

export type DropAction = "none" | "reschedule" | "move";

export type DropOutcome =
  | { kind: "none" }
  | { kind: "done" }
  | { kind: "partialMove"; newHref: string; sourceHref: string }
  | { kind: "error"; message: string };

/** Width of the edge band (px) that triggers week navigation while dragging. */
export const edgeZoneWidth = 48;
/** How long (ms) the pointer must dwell in an edge band before navigating. */
export const edgeDwellMs = 1000;
/** How long (ms) to wait after a navigation before the next dwell can start, so holding a bit too long after a jump costs at most one extra week instead of compounding immediately. */
export const edgeCooldownMs = 1000;

/** Decides between no-op, same-calendar reschedule, and cross-calendar move. */
export function decideDropAction(
  source: AppointmentDragPayload,
  target: DropCellTarget,
): DropAction {
  if (source.employeeRef !== target.employeeRef) {
    return "move";
  }
  if (source.date !== target.date) {
    return "reschedule";
  }
  return "none";
}

interface DropDeps {
  updateAssignment: (
    href: string,
    uid: string,
    date: string,
    projectRef: string,
    projectName: string,
  ) => Promise<
    { status: "ok"; data: null } | { status: "error"; error: string }
  >;
  moveAssignment: (
    href: string,
    targetEmployeeReference: string,
    date: string,
    projectRef: string,
    projectName: string,
  ) => Promise<
    | { status: "ok"; data: MoveAssignmentResult }
    | { status: "error"; error: string }
  >;
}

/**
 * Persists a drop: reschedule on the same calendar or move across calendars.
 * Both paths rebuild the VEVENT from the payload (fixed time window, single
 * daylite: description line); properties added in other calendar clients are
 * not preserved — lkr-planner owns its assignment events.
 */
export async function performDrop(
  source: AppointmentDragPayload,
  target: DropCellTarget,
  deps: DropDeps,
): Promise<DropOutcome> {
  const action = decideDropAction(source, target);

  if (action === "none") {
    return { kind: "none" };
  }

  if (action === "reschedule") {
    const result = await deps.updateAssignment(
      source.href,
      source.uid,
      target.date,
      source.projectRef,
      source.title,
    );
    if (result.status === "error") {
      return { kind: "error", message: result.error };
    }
    return { kind: "done" };
  }

  const result = await deps.moveAssignment(
    source.href,
    target.employeeRef,
    target.date,
    source.projectRef,
    source.title,
  );
  if (result.status === "error") {
    return { kind: "error", message: result.error };
  }
  if (result.data.kind === "sourceDeleteFailed") {
    return {
      kind: "partialMove",
      newHref: result.data.newHref,
      sourceHref: result.data.sourceHref,
    };
  }
  return { kind: "done" };
}

/** Returns which edge band of a container of the given width a pointer x is in. */
export function computeEdgeZone(
  x: number,
  width: number,
  band: number,
): "left" | "right" | null {
  if (x <= band) return "left";
  if (x >= width - band) return "right";
  return null;
}

/**
 * Owns the edge-hover dwell timer: entering an edge band starts the dwell, expiry
 * navigates one week, and after a cooldown the dwell restarts if the pointer is
 * still in the zone (multi-week jumps); leaving the band or stopping cancels
 * whichever phase (dwell or cooldown) is currently pending.
 */
export class EdgeHoverNavigator {
  private zone: "left" | "right" | null = null;
  private timer: ReturnType<typeof setTimeout> | null = null;

  constructor(
    private readonly onNavigate: (direction: -1 | 1) => void,
    private readonly dwellMs: number,
    private readonly cooldownMs: number,
  ) {}

  setZone(zone: "left" | "right" | null) {
    if (zone === this.zone) return;
    this.clearTimer();
    this.zone = zone;
    if (zone !== null) {
      this.startDwell(zone);
    }
  }

  stop() {
    this.clearTimer();
    this.zone = null;
  }

  private startDwell(zone: "left" | "right") {
    this.timer = setTimeout(() => {
      this.onNavigate(zone === "left" ? -1 : 1);
      this.startCooldown(zone);
    }, this.dwellMs);
  }

  private startCooldown(zone: "left" | "right") {
    this.timer = setTimeout(() => {
      // Restart while the pointer stays in the zone so one drag can cross several weeks.
      if (this.zone === zone) {
        this.startDwell(zone);
      }
    }, this.cooldownMs);
  }

  private clearTimer() {
    if (this.timer !== null) {
      clearTimeout(this.timer);
      this.timer = null;
    }
  }
}

export interface AppointmentDragState {
  /** Payload of the card currently being dragged, for the DragOverlay preview. */
  activePayload: AppointmentDragPayload | null;
  /** German error from the last failed drop; cleared when the next drag starts. */
  errorMessage: string | null;
  clearError: () => void;
  /** Set when a cross-employee move created the target but the source delete failed. */
  reconciliation: MoveReconciliation | null;
  resolveReconciliation: () => void;
  onDragStart: (event: DragStartEvent) => void;
  onDragMove: (event: DragMoveEvent) => void;
  onDragEnd: (event: DragEndEvent) => void;
  onDragCancel: () => void;
}

interface UseAppointmentDragArgs {
  onNavigateWeek: (direction: -1 | 1) => void;
  reloadAssignments: () => void;
}

/**
 * Wraps the dnd-kit drag lifecycle for the planning grid: carries the drag payload,
 * dispatches drops to the backend, drives edge-hover week navigation, and holds the
 * reconciliation state for a partial cross-employee move.
 */
export function useAppointmentDrag({
  onNavigateWeek,
  reloadAssignments,
}: UseAppointmentDragArgs): AppointmentDragState {
  const [activePayload, setActivePayload] =
    useState<AppointmentDragPayload | null>(null);
  // Source of truth for the drop: dnd-kit's `active.data` is a mutable ref tied to the
  // registered draggable, which unmounts when edge-hover navigation swaps the week, so
  // the payload captured at drag start must be used instead of re-reading it on drop.
  const activePayloadRef = useRef<AppointmentDragPayload | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [reconciliation, setReconciliation] =
    useState<MoveReconciliation | null>(null);

  // Latest-callback refs so drag handlers and the dwell timer never go stale.
  const onNavigateWeekRef = useRef(onNavigateWeek);
  onNavigateWeekRef.current = onNavigateWeek;
  const reloadAssignmentsRef = useRef(reloadAssignments);
  reloadAssignmentsRef.current = reloadAssignments;

  const navigatorRef = useRef<EdgeHoverNavigator | null>(null);
  if (navigatorRef.current === null) {
    navigatorRef.current = new EdgeHoverNavigator(
      (direction) => onNavigateWeekRef.current(direction),
      edgeDwellMs,
      edgeCooldownMs,
    );
  }

  useEffect(() => {
    return () => navigatorRef.current?.stop();
  }, []);

  const onDragStart = useCallback((event: DragStartEvent) => {
    const payload = event.active.data.current as
      | AppointmentDragPayload
      | undefined;
    activePayloadRef.current = payload ?? null;
    setActivePayload(payload ?? null);
    setErrorMessage(null);
  }, []);

  const onDragMove = useCallback((event: DragMoveEvent) => {
    const activator = event.activatorEvent as Partial<PointerEvent>;
    if (typeof activator.clientX !== "number") return;
    const pointerX = activator.clientX + event.delta.x;
    navigatorRef.current?.setZone(
      computeEdgeZone(pointerX, window.innerWidth, edgeZoneWidth),
    );
  }, []);

  const onDragEnd = useCallback((event: DragEndEvent) => {
    navigatorRef.current?.stop();
    setActivePayload(null);

    const source = activePayloadRef.current;
    activePayloadRef.current = null;
    const target = event.over?.data.current as DropCellTarget | undefined;
    if (!source || !target) return;

    void performDrop(source, target, {
      updateAssignment: (href, uid, date, projectRef, projectName) =>
        commands.updateAssignment({ href, uid, date, projectRef, projectName }),
      moveAssignment: commands.moveAssignment,
    })
      .then((outcome) => {
        if (outcome.kind === "done") {
          reloadAssignmentsRef.current();
          return;
        }
        if (outcome.kind === "partialMove") {
          setReconciliation({
            newHref: outcome.newHref,
            sourceHref: outcome.sourceHref,
          });
          return;
        }
        if (outcome.kind === "error") {
          setErrorMessage(outcome.message);
        }
      })
      // The generated bindings re-throw Error-typed rejections (IPC failures)
      // instead of returning a status object; without this the drop fails silently.
      .catch(() =>
        setErrorMessage("Der Einsatz konnte nicht verschoben werden."),
      );
  }, []);

  const onDragCancel = useCallback(() => {
    navigatorRef.current?.stop();
    activePayloadRef.current = null;
    setActivePayload(null);
  }, []);

  const clearError = useCallback(() => setErrorMessage(null), []);

  const resolveReconciliation = useCallback(() => {
    setReconciliation(null);
    reloadAssignmentsRef.current();
  }, []);

  return {
    activePayload,
    errorMessage,
    clearError,
    reconciliation,
    resolveReconciliation,
    onDragStart,
    onDragMove,
    onDragEnd,
    onDragCancel,
  };
}
