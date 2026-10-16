import { commands } from "../generated/tauri";

export async function openProjectInDaylite(
  projectRef: string,
): Promise<string | null> {
  const result = await commands
    .dayliteOpenProject(projectRef)
    .catch(() => null);

  return !result || result.status === "error"
    ? result?.error || "Das Projekt konnte in Daylite nicht geöffnet werden."
    : null;
}
