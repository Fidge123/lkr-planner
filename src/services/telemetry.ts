import { commands, type FrontendErrorInput } from "../generated/tauri";
import { unwrapCommandResult } from "./command-result";

const loadErrorMessage =
  "Die Diagnose-Einstellung konnte nicht geladen werden.";
const saveErrorMessage =
  "Die Diagnose-Einstellung konnte nicht gespeichert werden.";

export async function loadTelemetrySettings(): Promise<boolean> {
  return unwrapCommandResult(
    await commands.telemetryGetSettings(),
    loadErrorMessage,
  ).enabled;
}

export async function saveTelemetryEnabled(enabled: boolean): Promise<boolean> {
  return unwrapCommandResult(
    await commands.telemetrySetEnabled(enabled),
    saveErrorMessage,
  ).enabled;
}

/// Reporting an error must never produce a second error the user sees.
export async function reportFrontendError(
  error: FrontendErrorInput,
): Promise<void> {
  try {
    await commands.telemetryCaptureFrontendError(error);
  } catch {}
}
