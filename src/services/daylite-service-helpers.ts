import type { DayliteApiError } from "../generated/tauri";

export function normalizeOptionalString(
  value: string | null | undefined,
): string | undefined {
  if (typeof value !== "string") {
    return undefined;
  }

  const normalized = value.trim();
  return normalized.length > 0 ? normalized : undefined;
}

export function readDayliteApiErrorMessage(
  error: DayliteApiError | string,
  fallbackMessage: string,
): string {
  if (typeof error === "string") {
    return error;
  }

  if (typeof error.userMessage === "string" && error.userMessage.length > 0) {
    return error.userMessage;
  }

  return fallbackMessage;
}
