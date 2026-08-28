import { commands } from "../generated/tauri";

const openFailedMessage =
  "Das Projekt konnte in Daylite nicht geöffnet werden.";

/** Resolves to a German message when the project could not be opened, and to null when it was. */
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
