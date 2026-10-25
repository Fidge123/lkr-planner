import { commands } from "../generated/tauri";
import { callCommand } from "./command-result";

export type ProjectCategoryColors = Record<string, string>;

export function loadProjectCategoryColors(): Promise<ProjectCategoryColors> {
  return callCommand(
    commands.dayliteProjectCategoryColors(),
    "Die Kategoriefarben konnten nicht geladen werden.",
  );
}

export function projectCategoryColor(
  colors: ProjectCategoryColors,
  category: string | null | undefined,
): string | null {
  const name = category?.trim() ?? "";
  if (name.length === 0) return null;
  return colors[name] ?? null;
}
