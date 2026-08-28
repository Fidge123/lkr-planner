import { commands } from "../generated/tauri";

const openFailedMessage =
  "Das Projekt konnte in Daylite nicht geöffnet werden.";

/** The German failure message, or null when the project was opened. */
export async function openProjectInDaylite(
  projectRef: string,
): Promise<string | null> {
  const result = await commands
    .dayliteOpenProject(projectRef)
    .catch(() => null);
  if (!result || result.status === "error") {
    return result?.error || openFailedMessage;
  }

  return null;
}
