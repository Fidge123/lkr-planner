import { commands } from "../generated/tauri";

export type ProjectCategoryColors = Record<string, string>;

let inFlight: Promise<ProjectCategoryColors> | null = null;
const listeners = new Set<() => void>();

/** Held for the whole session, unlike the TTL-cached project and contact reads. */
export function loadProjectCategoryColors(): Promise<ProjectCategoryColors> {
  inFlight ??= fetchProjectCategoryColors();
  return inFlight;
}

export function resetProjectCategoryColors(): void {
  inFlight = null;
  for (const listener of listeners) listener();
}

export function subscribeProjectCategoryColors(
  listener: () => void,
): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

export function projectCategoryColor(
  colors: ProjectCategoryColors,
  category: string | null | undefined,
): string | null {
  const name = category?.trim() ?? "";
  if (name.length === 0) return null;
  return colors[name] ?? null;
}

async function fetchProjectCategoryColors(): Promise<ProjectCategoryColors> {
  const result = await commands
    .dayliteProjectCategoryColors()
    .catch(() => null);
  if (!result || result.status === "error") {
    // A missed color is cosmetic, so the failure is not surfaced; dropping the
    // promise lets the next opened modal try again.
    inFlight = null;
    return {};
  }

  return result.data;
}
