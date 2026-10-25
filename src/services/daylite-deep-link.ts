import { commands } from "../generated/tauri";
import { callCommand } from "./command-result";

/** Resolves with the German error message, or null once the project is open. */
export async function openProjectInDaylite(
  projectRef: string,
): Promise<string | null> {
  try {
    await callCommand(
      commands.dayliteOpenProject(projectRef),
      "Das Projekt konnte in Daylite nicht geöffnet werden.",
    );
    return null;
  } catch (error) {
    return (error as Error).message;
  }
}
