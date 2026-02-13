import { commands, type HealthStatus } from "../generated/tauri";

export async function checkHealth(): Promise<HealthStatus> {
  try {
    const result = await commands.checkHealth();
    return result;
  } catch (error) {
    throw new Error(
      `Health check fehlgeschlagen: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
}
