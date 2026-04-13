import type { CellEvent } from "../types";

export function TimetableCell({ highlight = false, events }: Props) {
  return (
    <td className={`align-top p-2 ${highlight ? "bg-primary/10" : ""}`}>
      <ul className="flex flex-col gap-1 list-none">
        {events.map((event) =>
          event.kind === "bare" ? (
            <li key={event.uid}>
              <span
                className={`flex items-center w-full gap-2 p-2 rounded-lg cursor-default text-base-content transition-colors hover:bg-base-300 ${event.color}`}
              >
                <EventTime
                  startTime={event.startTime}
                  endTime={event.endTime}
                />
                <h4 className="truncate flex-1 font-normal text-sm">
                  {event.title}
                </h4>
              </span>
            </li>
          ) : (
            <li key={event.uid}>
              <button
                type="button"
                className={`btn btn-block text-base-100 p-2 rounded-lg transition-all hover:brightness-90 active:brightness-75 ${event.color}`}
              >
                <EventTime
                  startTime={event.startTime}
                  endTime={event.endTime}
                />
                <h4 className="truncate flex-1 font-medium">{event.title}</h4>
              </button>
            </li>
          ),
        )}
        <li>
          <button
            type="button"
            className="btn btn-dash btn-block rounded-lg opacity-20 hover:opacity-80 transition-opacity"
            aria-label="Aufgabe hinzufügen"
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
  events: CellEvent[];
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
