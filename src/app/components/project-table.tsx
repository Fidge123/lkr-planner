import type { PlanningProjectRecord } from "../../generated/tauri";

export function ProjectTable({ projects, isLoading }: Props) {
  return (
    <section className="p-4 border-t border-base-300">
      <h2 className="text-lg font-semibold">Geladene Projekte</h2>
      {isLoading ? (
        <p className="mt-2 text-base-content/70">Projekte werden geladen...</p>
      ) : null}
      {!isLoading && projects.length === 0 ? (
        <p className="mt-2 text-base-content/70">Keine Projekte gefunden</p>
      ) : null}
      {!isLoading && projects.length > 0 ? (
        <table className="table table-sm mt-3">
          <thead>
            <tr>
              <th>Projekt</th>
              <th>Status</th>
              <th>Fällig</th>
            </tr>
          </thead>
          <tbody>
            {projects.map((project, index) => (
              <tr key={buildProjectRowKey(project, index)}>
                <td>{project.name}</td>
                <td>{toGermanProjectStatus(project.status)}</td>
                <td>{formatGermanDate(project.due)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      ) : null}
    </section>
  );
}

interface Props {
  projects: PlanningProjectRecord[];
  isLoading: boolean;
}

function toGermanProjectStatus(
  status: PlanningProjectRecord["status"],
): string {
  if (status === "new_status") {
    return "Neu";
  }
  if (status === "in_progress") {
    return "In Arbeit";
  }
  if (status === "done") {
    return "Erledigt";
  }
  if (status === "abandoned") {
    return "Abgebrochen";
  }
  if (status === "cancelled") {
    return "Storniert";
  }
  if (status === "deferred") {
    return "Zurückgestellt";
  }

  return "Unbekannt";
}

function formatGermanDate(isoDate: string | null | undefined): string {
  if (!isoDate) {
    return "Kein Termin";
  }

  const date = new Date(isoDate);
  if (Number.isNaN(date.getTime())) {
    return "Kein Termin";
  }

  return date.toLocaleDateString("de-DE", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  });
}

function buildProjectRowKey(
  project: PlanningProjectRecord,
  index: number,
): string {
  const stableReference =
    typeof project.self === "string" ? project.self.trim() : "";
  if (stableReference.length > 0) {
    return stableReference;
  }

  const stableName = project.name.trim();
  if (stableName.length > 0) {
    return `project-${stableName}-${index}`;
  }

  return `project-empty-${index}`;
}
