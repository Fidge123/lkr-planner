import { commands, type HealthStatus } from "../generated/tauri";
import { unwrapCommandResult } from "./daylite-service-helpers";

export async function checkHealth(): Promise<HealthStatus> {
  try {
    return unwrapCommandResult(
      await commands.checkHealth(),
      "Health check fehlgeschlagen: unbekannter Fehler",
    );
  } catch (error) {
    throw new Error(
      `Health check fehlgeschlagen: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
}
