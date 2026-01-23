import type { WorkItem } from "../../types";

export function TimetableCell({ highlight = false, projects }: Props) {
  return (
    <td className={`align-top p-2 ${highlight ? "bg-primary/10" : ""}`}>
      <ul className="flex flex-col gap-1 list-none">
        {projects.map((workItem) => (
          <li key={workItem.id}>
            <button
              type="button"
              className={`btn btn-block text-base-100 p-2 rounded-lg ${workItem.color}`}
            >
              <h4 className="truncate flex-1 font-medium">{workItem.title}</h4>
            </button>
          </li>
        ))}
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
  projects: WorkItem[];
}
