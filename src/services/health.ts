import { commands, type HealthStatus } from "../generated/tauri";

export async function checkHealth(): Promise<HealthStatus> {
  try {
    const result = await commands.checkHealth();
    if (result.status === "ok") {
      return result.data;
    }

    throw new Error(result.error);
  } catch (error) {
    throw new Error(
      `Health check fehlgeschlagen: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
}
