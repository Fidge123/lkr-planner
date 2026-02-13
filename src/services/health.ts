import { invoke } from "@tauri-apps/api/core";

/**
 * Service layer for health/status checks.
 * This demonstrates the architecture pattern:
 * - React UI consumes service functions
 * - Services invoke Tauri commands
 * - Network and secrets are handled in Rust
 */

export interface HealthStatus {
  status: "healthy" | "unhealthy";
  timestamp: string;
  version: string;
}

/**
 * Check the health status of the application backend.
 * @returns Promise with health status information
 * @throws Error if the health check fails
 */
export async function checkHealth(): Promise<HealthStatus> {
  try {
    const result = await invoke<HealthStatus>("check_health");
    return result;
  } catch (error) {
    throw new Error(
      `Health check fehlgeschlagen: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
}
