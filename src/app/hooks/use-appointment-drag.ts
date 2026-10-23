import type {
  DragEndEvent,
  DragMoveEvent,
  DragStartEvent,
} from "@dnd-kit/core";
import { useCallback, useEffect, useRef, useState } from "react";
import {
  moveAssignment,
  reorderAssignment,
  updateAssignment,
} from "../../services/assignments";
import type { MoveReconciliation } from "../components/move-reconciliation-dialog";
import { useWeekCache } from "./use-planning-assignments";

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

export interface CardRect {
  uid: string;
  top: number;
  height: number;
}

/**
 * The cell is the only drop zone; the position within it comes from card geometry.
 * An inserted preview therefore never becomes a target, and the cell keeps its highlight.
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
   * Insertion position among the target cell's assignments, excluding the dragged card.
   * Equals the number of those cards when the drop appends.
   */
  orderIndex: number;
}

export type DropAction = "none" | "reorder" | "reschedule" | "move";

/**
 * The index counts the cards the pointer has passed, so a card's upper half lands before it.
 * A preview only moves cards below the pointer, so the index stays stable and cannot oscillate.
 */
export function insertionIndexAt(cards: CardRect[], pointerY: number): number {
  return cards.filter((card) => card.top + card.height / 2 < pointerY).length;
}

export function resolveDropTarget(
  source: AppointmentDragPayload,
  zone: DropZoneData,
  pointerY: number,
): DropCellTarget {
  // The backend indexes among the day's other assignments, so the dragged card is excluded.
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

const edgeZoneWidth = 48;
const edgeDwellMs = 1000;
// Without a cooldown the dwell restarts as soon as it fires, compounding into several weeks after a jump.
const edgeCooldownMs = 1000;

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

const assignmentWrites = {
  updateAssignment,
  reorderAssignment,
  moveAssignment,
};

// Both paths rebuild the VEVENT from the payload, dropping properties set by other calendar clients.
export async function performDrop(
  source: AppointmentDragPayload,
  target: DropCellTarget,
  deps: typeof assignmentWrites = assignmentWrites,
): Promise<DropOutcome> {
  const action = decideDropAction(source, target);

  if (action === "none") {
    return { kind: "none" };
  }

  try {
    if (action === "reorder") {
      await deps.reorderAssignment({
        href: source.href,
        uid: source.uid,
        date: source.date,
        orderIndex: target.orderIndex,
      });
      return { kind: "done" };
    }

    if (action === "reschedule") {
      await deps.updateAssignment({
        href: source.href,
        uid: source.uid,
        date: target.date,
        projectRef: source.projectRef,
        projectName: source.title,
        orderIndex: target.orderIndex,
        // A drag is not a deliberate enough gesture to carry a protection override.
        overrideProtection: false,
      });
      return { kind: "done" };
    }

    const moved = await deps.moveAssignment({
      href: source.href,
      targetEmployeeReference: target.employeeRef,
      date: target.date,
      projectRef: source.projectRef,
      projectName: source.title,
      orderIndex: target.orderIndex,
    });
    return moved.kind === "sourceDeleteFailed"
      ? {
          kind: "partialMove",
          newHref: moved.newHref,
          sourceHref: moved.sourceHref,
        }
      : { kind: "done" };
  } catch (error) {
    return { kind: "error", message: (error as Error).message };
  }
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
  /** The displayed week, which a drop edits. */
  weekStart: string;
  onNavigateWeek: (direction: -1 | 1) => void;
}

export function useAppointmentDrag({
  weekStart,
  onNavigateWeek,
}: UseAppointmentDragArgs): AppointmentDragState {
  const cache = useWeekCache();
  const [activePayload, setActivePayload] =
    useState<AppointmentDragPayload | null>(null);
  // `active.data` is a mutable ref that unmounts when the drag leaves the week, so capture it at drag start.
  const activePayloadRef = useRef<AppointmentDragPayload | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [dropPreview, setDropPreview] = useState<DropPreview | null>(null);
  const [reconciliation, setReconciliation] =
    useState<MoveReconciliation | null>(null);

  // dnd-kit's `delta` is `scrollAdjustedTranslate`, while card rects are viewport-relative.
  // Deriving the cursor from it would count auto-scroll twice, so the real pointer is tracked instead.
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
  const weekStartRef = useRef(weekStart);
  weekStartRef.current = weekStart;

  // The reconciliation path reloads after the dialog closes, long after the drop cleared activePayloadRef.
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
      if (decideDropAction(source, target) === "none") return;

      // A rejected write is undone by the same refresh that confirms a successful one.
      const editedWeek = weekStartRef.current;
      cache.moveEvent(editedWeek, source.uid, target);
      const refreshAffectedRows = () => {
        void cache.refreshEmployees(editedWeek, [
          ...new Set([source.employeeRef, target.employeeRef]),
        ]);
      };

      void performDrop(source, target).then((outcome) => {
        if (outcome.kind === "partialMove") {
          droppedUidRef.current = source.uid;
          setReconciliation({
            newHref: outcome.newHref,
            sourceHref: outcome.sourceHref,
          });
        }
        if (outcome.kind === "error") {
          setErrorMessage(outcome.message);
        }
        refreshAffectedRows();
      });
    },
    [cache, targetOf],
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
      cache.dropWeeksContaining(droppedUidRef.current);
      droppedUidRef.current = null;
    }
    void cache.reload(weekStartRef.current);
  }, [cache]);

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
