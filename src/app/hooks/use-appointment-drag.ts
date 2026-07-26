import type {
  DragEndEvent,
  DragMoveEvent,
  DragStartEvent,
} from "@dnd-kit/core";
import { useCallback, useEffect, useRef, useState } from "react";
import { commands, type MoveAssignmentResult } from "../../generated/tauri";
import type { MoveReconciliation } from "../components/move-reconciliation-dialog";

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

export const edgeZoneWidth = 48;
export const edgeDwellMs = 1000;
// Without a cooldown the dwell restarts the instant it fires, so holding a
// moment too long after a jump compounds into several weeks at once.
export const edgeCooldownMs = 1000;

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

// Both paths rebuild the VEVENT from the payload, so properties added in other
// calendar clients are not preserved.
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

export function computeEdgeZone(
  x: number,
  width: number,
  band: number,
): "left" | "right" | null {
  if (x <= band) return "left";
  if (x >= width - band) return "right";
  return null;
}

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
  activePayload: AppointmentDragPayload | null;
  errorMessage: string | null;
  clearError: () => void;
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
  invalidateWeeksContaining: (uid: string) => void;
}

export function useAppointmentDrag({
  onNavigateWeek,
  reloadAssignments,
  invalidateWeeksContaining,
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

  const onNavigateWeekRef = useRef(onNavigateWeek);
  onNavigateWeekRef.current = onNavigateWeek;
  const reloadAssignmentsRef = useRef(reloadAssignments);
  reloadAssignmentsRef.current = reloadAssignments;
  const invalidateWeeksContainingRef = useRef(invalidateWeeksContaining);
  invalidateWeeksContainingRef.current = invalidateWeeksContaining;

  // Held for the reconciliation path, which reloads only once the user resolves
  // the dialog, long after the drop cleared activePayloadRef.
  const droppedUidRef = useRef<string | null>(null);

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
          invalidateWeeksContainingRef.current(source.uid);
          reloadAssignmentsRef.current();
          return;
        }
        if (outcome.kind === "partialMove") {
          droppedUidRef.current = source.uid;
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
    if (droppedUidRef.current !== null) {
      invalidateWeeksContainingRef.current(droppedUidRef.current);
      droppedUidRef.current = null;
    }
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
