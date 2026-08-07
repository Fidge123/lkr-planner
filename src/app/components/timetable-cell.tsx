import { useDraggable, useDroppable } from "@dnd-kit/core";
import { TriangleAlert } from "lucide-react";
import type { ReactNode } from "react";
import { useRef } from "react";
import { assignmentPositions, sortCellEvents } from "../cell-order";
import type {
  AppointmentDragPayload,
  CardRect,
  DropPreview,
} from "../hooks/use-appointment-drag";
import type { GhostSuggestion } from "../next-day-quick-add";
import {
  type CellEvent,
  hasAbsenceConflict,
  isUnresolvedAssignment,
} from "../types";

export function TimetableCell({
  highlight = false,
  isHoliday = false,
  employeeRef = "",
  date = "",
  events,
  suggestion,
  dropPreview = null,
  draggedUid = null,
  onAddClick,
  onEventClick,
  onSuggestionClick,
}: Props) {
  const orderedEvents = sortCellEvents(events);
  const positions = assignmentPositions(orderedEvents);
  const conflict = hasAbsenceConflict(events);

  // Measured on demand rather than tracked in state: the drop position is only ever read
  // mid-drag, and rects go stale as soon as a row above this one changes height.
  const cardNodes = useRef(new Map<string, HTMLLIElement>());
  const { isOver, setNodeRef } = useDroppable({
    id: `cell-${employeeRef}-${date}`,
    data: {
      kind: "cell",
      employeeRef,
      date,
      cardRects: () =>
        [...positions.keys()].flatMap((uid) => {
          const rect = cardNodes.current.get(uid)?.getBoundingClientRect();
          // A card lifted out of the flow measures as a zero rect, which would otherwise
          // sort ahead of every real card.
          if (!rect || rect.height === 0) return [];
          return [{ uid, top: rect.top, height: rect.height }];
        }) satisfies CardRect[],
    },
  });

  const preview =
    dropPreview &&
    dropPreview.employeeRef === employeeRef &&
    dropPreview.date === date
      ? dropPreview
      : null;

  return (
    <td
      ref={setNodeRef}
      className={cellClass(highlight, isHoliday, isOver, conflict)}
    >
      <ul className="flex flex-col gap-1 list-none">
        {conflict ? (
          <li className="flex items-center gap-1 text-error text-xs font-medium">
            <TriangleAlert className="size-4 shrink-0" aria-hidden="true" />
            Abwesenheit und Termin am selben Tag
          </li>
        ) : null}
        {withDropPreview(orderedEvents, preview, draggedUid, (event) =>
          event.kind === "absence" ? (
            <li key={event.uid}>
              <span
                className={`flex items-center w-full gap-4 p-2 rounded-lg cursor-default text-base-content transition-colors ${event.color}`}
              >
                <h4 className="flex-1 min-w-0 font-normal text-sm italic">
                  {event.title}
                </h4>
              </span>
            </li>
          ) : event.kind === "bare" ? (
            <li key={event.uid}>
              <span
                className={`flex items-center w-full gap-4 p-2 rounded-lg cursor-default text-base-content transition-colors hover:bg-base-300 ${event.color}`}
              >
                <EventTime
                  startTime={event.startTime}
                  endTime={event.endTime}
                />
                <h4 className="flex-1 min-w-0 font-normal text-sm">
                  {event.title}
                </h4>
              </span>
            </li>
          ) : (
            <li
              key={event.uid}
              // Collapsed while in flight so the drop preview replaces the card instead of
              // adding to the cell's height: a cell that grows mid-drag pushes the rows
              // under it away from the pointer. The card itself stays laid out, because
              // dnd-kit measures it to position the drag overlay.
              className={
                event.uid === draggedUid
                  ? "relative h-0 overflow-visible"
                  : undefined
              }
              ref={(node) => {
                if (node) cardNodes.current.set(event.uid, node);
                else cardNodes.current.delete(event.uid);
              }}
            >
              <DraggableAssignmentCard
                event={event}
                employeeRef={employeeRef}
                date={date}
                position={positions.get(event.uid) ?? 0}
                lifted={event.uid === draggedUid}
                onEventClick={onEventClick}
              />
            </li>
          ),
        )}
        {suggestion ? (
          <li>
            <button
              type="button"
              className="btn btn-block h-auto justify-start gap-4 p-2 rounded-lg border-2 border-dashed border-base-content/40 bg-transparent text-base-content opacity-50 transition-opacity hover:opacity-80"
              aria-label={`Vorschlag übernehmen: ${suggestion.projectName}`}
              onClick={() => onSuggestionClick?.(suggestion)}
            >
              <h4 className="flex-1 min-w-0 font-medium">
                {suggestion.projectName}
              </h4>
            </button>
          </li>
        ) : null}
        <li>
          <button
            type="button"
            className="btn btn-dash btn-block rounded-lg opacity-20 hover:opacity-80 transition-opacity"
            aria-label="Aufgabe hinzufügen"
            onClick={onAddClick}
          >
            +
          </button>
        </li>
      </ul>
    </td>
  );
}

/**
 * Splices the drop preview into the cell's rendered items, before the first assignment that
 * already has `orderIndex` other assignments ahead of it. The dragged card is skipped in that
 * count because it does not occupy a position in the day it is being dropped into.
 */
function withDropPreview(
  events: CellEvent[],
  preview: DropPreview | null,
  draggedUid: string | null,
  renderEvent: (event: CellEvent) => ReactNode,
): ReactNode[] {
  const items: ReactNode[] = [];
  let othersAhead = 0;
  let placed = false;
  for (const event of events) {
    if (event.kind === "assignment") {
      if (preview && !placed && othersAhead === preview.orderIndex) {
        items.push(
          <DropPreviewCard key="drop-preview" title={preview.title} />,
        );
        placed = true;
      }
      if (event.uid !== draggedUid) othersAhead += 1;
    }
    items.push(renderEvent(event));
  }
  if (preview && !placed) {
    items.push(<DropPreviewCard key="drop-preview" title={preview.title} />);
  }
  return items;
}

function DropPreviewCard({ title }: { title: string }) {
  return (
    <li aria-hidden="true">
      <span
        data-drop-preview="true"
        className={`${assignmentCardClass} h-[3.25rem] border-2 border-dashed border-primary bg-primary/10 text-base-content/70 pointer-events-none`}
      >
        <h4 className="flex-1 min-w-0 font-medium">{title}</h4>
      </span>
    </li>
  );
}

interface Props {
  highlight: boolean;
  isHoliday?: boolean;
  employeeRef?: string;
  date?: string;
  events: CellEvent[];
  suggestion?: GhostSuggestion;
  /** Where the in-flight drag would land; null unless this cell is the target. */
  dropPreview?: DropPreview | null;
  /** UID of the card being dragged, so it is not counted as its own neighbour. */
  draggedUid?: string | null;
  onAddClick: () => void;
  onEventClick: (event: CellEvent) => void;
  onSuggestionClick?: (suggestion: GhostSuggestion) => void;
}

export const assignmentCardClass =
  "flex items-center w-full gap-4 p-2 rounded-lg";

/** Width and default color of the strip live in `assignmentStripClass`; the Daylite
 *  color is passed through verbatim so any CSS color notation it uses still works. */
export const assignmentStripClass = "border-l-4 border-base-content/30";

export function categoryStrip(
  categoryColor: string | null,
): { borderLeftColor: string } | undefined {
  return categoryColor ? { borderLeftColor: categoryColor } : undefined;
}

export function AssignmentCardBody({
  startTime,
  endTime,
  title,
  isUnresolved = false,
}: BodyProps) {
  return (
    <>
      <EventTime startTime={startTime} endTime={endTime} />
      {isUnresolved ? (
        <TriangleAlert
          className="size-4 shrink-0 text-error"
          aria-hidden="true"
        />
      ) : null}
      <h4 className="flex-1 min-w-0 font-medium">
        {isUnresolved ? (
          <>
            <em className="font-normal opacity-70">Beschreibung für </em>
            {title}
            <em className="font-normal opacity-70">
              {" "}
              konnte nicht abgerufen werden
            </em>
          </>
        ) : (
          title
        )}
      </h4>
    </>
  );
}

interface BodyProps {
  startTime: string | null;
  endTime: string | null;
  title: string;
  /** Wraps the calendar summary in the German "could not be read" note. */
  isUnresolved?: boolean;
}

function DraggableAssignmentCard({
  event,
  employeeRef,
  date,
  position,
  lifted = false,
  onEventClick,
}: CardProps) {
  const payload: AppointmentDragPayload = {
    uid: event.uid,
    href: event.href ?? "",
    projectRef: event.projectRef ?? "",
    employeeRef,
    date,
    title: event.title,
    color: event.color,
    categoryColor: event.categoryColor,
    position,
  };
  const unresolved = isUnresolvedAssignment(event);
  const canDrag = Boolean(event.href) && !unresolved;
  const { attributes, listeners, setNodeRef } = useDraggable({
    id: `assignment-${employeeRef}-${event.uid}`,
    data: payload,
    disabled: !canDrag,
  });

  return (
    <button
      ref={setNodeRef}
      type="button"
      className={`btn btn-block h-auto justify-start ${assignmentCardClass} ${assignmentStripClass} text-base-content transition-[filter] hover:brightness-90 active:brightness-75 ${event.color} ${lifted ? "absolute inset-x-0 top-0 invisible pointer-events-none" : ""}`}
      style={categoryStrip(event.categoryColor)}
      onClick={() => onEventClick(event)}
      {...(canDrag ? { ...listeners, ...attributes } : {})}
    >
      <AssignmentCardBody
        startTime={event.startTime}
        endTime={event.endTime}
        title={event.title}
        isUnresolved={unresolved}
      />
    </button>
  );
}

interface CardProps {
  event: CellEvent;
  employeeRef: string;
  date: string;
  position: number;
  /** Out of the cell's flow while this card is the one being dragged. */
  lifted?: boolean;
  onEventClick: (event: CellEvent) => void;
}

function EventTime({ startTime, endTime }: TimeProps) {
  if (!startTime) return null;
  return (
    <div className="flex flex-col text-xs leading-tight shrink-0 opacity-70 tabular-nums">
      <span>{startTime}</span>
      {endTime && <span>{endTime}</span>}
    </div>
  );
}

interface TimeProps {
  startTime: string | null;
  endTime: string | null;
}

function cellClass(
  highlight: boolean,
  isHoliday: boolean,
  isDropTarget: boolean,
  conflict: boolean,
): string {
  const base = isHoliday
    ? "align-top p-2 bg-base-200/60"
    : highlight
      ? "align-top p-2 bg-primary/10"
      : "align-top p-2";
  if (isDropTarget) return `${base} ring-2 ring-inset ring-primary`;
  return conflict ? `${base} ring-2 ring-inset ring-error` : base;
}
