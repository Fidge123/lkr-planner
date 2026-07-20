import { useDraggable, useDroppable } from "@dnd-kit/core";
import type { AppointmentDragPayload } from "../hooks/use-appointment-drag";
import type { GhostSuggestion } from "../next-day-quick-add";
import type { CellEvent } from "../types";

export function TimetableCell({
  highlight = false,
  isHoliday = false,
  employeeRef = "",
  date = "",
  events,
  suggestion,
  onAddClick,
  onEventClick,
  onSuggestionClick,
}: Props) {
  const { isOver, setNodeRef } = useDroppable({
    id: `cell-${employeeRef}-${date}`,
    data: { employeeRef, date },
  });

  return (
    <td ref={setNodeRef} className={cellClass(highlight, isHoliday, isOver)}>
      <ul className="flex flex-col gap-1 list-none">
        {events.map((event) =>
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
            <li key={event.uid}>
              <DraggableAssignmentCard
                event={event}
                employeeRef={employeeRef}
                date={date}
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

interface Props {
  highlight: boolean;
  isHoliday?: boolean;
  /** Daylite reference of the row's employee; source/target identity for drag-and-drop. */
  employeeRef?: string;
  /** ISO date (yyyy-MM-dd) of this day cell; source/target identity for drag-and-drop. */
  date?: string;
  events: CellEvent[];
  suggestion?: GhostSuggestion;
  onAddClick: () => void;
  onEventClick: (event: CellEvent) => void;
  onSuggestionClick?: (suggestion: GhostSuggestion) => void;
}

/** An assignment card that can be dragged to another day or employee cell. */
function DraggableAssignmentCard({
  event,
  employeeRef,
  date,
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
  };
  // An unresolved project renders a German error placeholder as the title;
  // dropping such a card would persist that placeholder as the event summary.
  const unresolved = event.projectRef !== null && !event.projectStatus;
  const canDrag = Boolean(event.href) && !unresolved;
  const { attributes, listeners, setNodeRef, isDragging } = useDraggable({
    id: `assignment-${employeeRef}-${event.uid}`,
    data: payload,
    disabled: !canDrag,
  });

  return (
    <button
      ref={setNodeRef}
      type="button"
      className={`btn btn-block h-auto justify-start gap-4 text-base-100 p-2 rounded-lg transition-all hover:brightness-90 active:brightness-75 ${event.color} ${isDragging ? "opacity-40" : ""}`}
      onClick={() => onEventClick(event)}
      {...(canDrag ? { ...listeners, ...attributes } : {})}
    >
      <EventTime startTime={event.startTime} endTime={event.endTime} />
      <h4 className="flex-1 min-w-0 font-medium">{event.title}</h4>
    </button>
  );
}

interface CardProps {
  event: CellEvent;
  employeeRef: string;
  date: string;
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
): string {
  const base = isHoliday
    ? "align-top p-2 bg-base-200/60"
    : highlight
      ? "align-top p-2 bg-primary/10"
      : "align-top p-2";
  return isDropTarget ? `${base} ring-2 ring-inset ring-primary` : base;
}
