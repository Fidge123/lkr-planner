import type { CellEvent } from "../types";

export function TimetableCell({ highlight = false, events }: Props) {
  return (
    <td className={`align-top p-2 ${highlight ? "bg-primary/10" : ""}`}>
      <ul className="flex flex-col gap-1 list-none">
        {events.map((event) =>
          event.kind === "bare" ? (
            <li key={event.uid}>
              <span className="btn btn-block btn-ghost text-base-content/50 p-2 rounded-lg cursor-default">
                <h4 className="truncate flex-1 font-normal text-sm">
                  {event.title}
                </h4>
              </span>
            </li>
          ) : (
            <li key={event.uid}>
              <button
                type="button"
                className={`btn btn-block text-base-100 p-2 rounded-lg ${event.color}`}
              >
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
