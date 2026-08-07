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
  categoryColor: string | null;
  /** Rendered position among its own cell's assignments. */
  position: number;
}

/** Live geometry of one rendered assignment card. */
export interface CardRect {
  uid: string;
  top: number;
  height: number;
}

/**
 * Droppable payload of a day cell. The cell is the only drop zone: the position inside it
 * comes from the cards' geometry, so an inserted drop preview never becomes a target of its
 * own and the cell keeps its highlight wherever the pointer sits within it.
 */
export interface DropZoneData {
  kind: "cell";
  employeeRef: string;
  date: string;
  /** Rects of the cell's assignment cards, in rendered order, measured on demand. */
  cardRects: () => CardRect[];
}

export interface DropCellTarget {
  employeeRef: string;
  date: string;
  /**
   * Insertion position among the target cell's assignments, counted without the dragged
   * card. Equal to the number of those cards when the drop appends.
   */
  orderIndex: number;
}

export type DropAction = "none" | "reorder" | "reschedule" | "move";

/**
 * The insertion position is the number of cards the pointer has already passed, so a pointer
 * in a card's upper half lands before it and one in its lower half lands after it.
 *
 * Inserting a preview at this index only moves cards that already sit below the pointer, so
 * the index it yields does not change once the preview is on screen and cannot oscillate.
 */
export function insertionIndexAt(cards: CardRect[], pointerY: number): number {
  return cards.filter((card) => card.top + card.height / 2 < pointerY).length;
}

export function resolveDropTarget(
  source: AppointmentDragPayload,
  zone: DropZoneData,
  pointerY: number,
): DropCellTarget {
  // The dragged card is never its own neighbour: the backend positions it among the day's
  // other assignments, so excluding it here makes the index directly the persisted one.
  const others = zone.cardRects().filter((card) => card.uid !== source.uid);
  return {
    employeeRef: zone.employeeRef,
    date: zone.date,
    orderIndex: insertionIndexAt(others, pointerY),
  };
}

function isSameTarget(
  a: DropCellTarget | null,
  b: DropCellTarget | null,
): boolean {
  if (a === null || b === null) return a === b;
  return (
    a.employeeRef === b.employeeRef &&
    a.date === b.date &&
    a.orderIndex === b.orderIndex
  );
}

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
  if (target.orderIndex !== source.position) {
    return "reorder";
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
    orderIndex: number,
  ) => Promise<
    { status: "ok"; data: null } | { status: "error"; error: string }
  >;
  reorderAssignment: (
    href: string,
    uid: string,
    date: string,
    orderIndex: number,
  ) => Promise<
    { status: "ok"; data: null } | { status: "error"; error: string }
  >;
  moveAssignment: (
    href: string,
    targetEmployeeReference: string,
    date: string,
    projectRef: string,
    projectName: string,
    orderIndex: number,
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

  if (action === "reorder") {
    const result = await deps.reorderAssignment(
      source.href,
      source.uid,
      source.date,
      target.orderIndex,
    );
    if (result.status === "error") {
      return { kind: "error", message: result.error };
    }
    return { kind: "done" };
  }

  if (action === "reschedule") {
    const result = await deps.updateAssignment(
      source.href,
      source.uid,
      target.date,
      source.projectRef,
      source.title,
      target.orderIndex,
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
    target.orderIndex,
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

/** Where a drop would land right now, and what the card being dragged is called. */
export interface DropPreview extends DropCellTarget {
  title: string;
}

export interface AppointmentDragState {
  activePayload: AppointmentDragPayload | null;
  dropPreview: DropPreview | null;
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
  const [dropPreview, setDropPreview] = useState<DropPreview | null>(null);
  const [reconciliation, setReconciliation] =
    useState<MoveReconciliation | null>(null);

  // dnd-kit's `delta` is `scrollAdjustedTranslate`, i.e. the translation plus the scrolling
  // that happened since the drag started, while card rects are viewport-relative. Deriving
  // the cursor from it would count auto-scroll twice, so the real pointer is tracked instead.
  const pointerRef = useRef<{ x: number; y: number } | null>(null);
  useEffect(() => {
    const track = (event: PointerEvent) => {
      pointerRef.current = { x: event.clientX, y: event.clientY };
    };
    window.addEventListener("pointermove", track, { passive: true });
    return () => window.removeEventListener("pointermove", track);
  }, []);

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

  const targetOf = useCallback(
    (event: DragMoveEvent | DragEndEvent): DropCellTarget | null => {
      const source = activePayloadRef.current;
      const zone = event.over?.data.current as DropZoneData | undefined;
      const pointer = pointerRef.current;
      if (!source || !zone || !pointer) return null;
      return resolveDropTarget(source, zone, pointer.y);
    },
    [],
  );

  const onDragStart = useCallback((event: DragStartEvent) => {
    const activator = event.activatorEvent as Partial<PointerEvent>;
    if (
      typeof activator.clientX === "number" &&
      typeof activator.clientY === "number"
    ) {
      pointerRef.current = { x: activator.clientX, y: activator.clientY };
    }
    const payload = event.active.data.current as
      | AppointmentDragPayload
      | undefined;
    activePayloadRef.current = payload ?? null;
    setActivePayload(payload ?? null);
    setErrorMessage(null);
  }, []);

  const onDragMove = useCallback(
    (event: DragMoveEvent) => {
      const pointer = pointerRef.current;
      if (pointer) {
        navigatorRef.current?.setZone(
          computeEdgeZone(pointer.x, window.innerWidth, edgeZoneWidth),
        );
      }

      const target = targetOf(event);
      const title = activePayloadRef.current?.title ?? "";
      // Every pointer move fires this, so an unchanged preview must not re-render the grid.
      setDropPreview((current) =>
        isSameTarget(current, target)
          ? current
          : target === null
            ? null
            : { ...target, title },
      );
    },
    [targetOf],
  );

  const onDragEnd = useCallback(
    (event: DragEndEvent) => {
      navigatorRef.current?.stop();
      setActivePayload(null);
      setDropPreview(null);

      const source = activePayloadRef.current;
      const target = targetOf(event);
      activePayloadRef.current = null;
      if (!source || !target) return;

      void performDrop(source, target, {
        updateAssignment: (
          href,
          uid,
          date,
          projectRef,
          projectName,
          orderIndex,
        ) =>
          commands.updateAssignment({
            href,
            uid,
            date,
            projectRef,
            projectName,
            orderIndex,
          }),
        reorderAssignment: commands.reorderAssignment,
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
    },
    [targetOf],
  );

  const onDragCancel = useCallback(() => {
    navigatorRef.current?.stop();
    activePayloadRef.current = null;
    setActivePayload(null);
    setDropPreview(null);
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
    dropPreview,
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
