import type { AppError } from "../generated/tauri";

type CommandResult<T> =
  | { status: "ok"; data: T }
  | { status: "error"; error: AppError };

function unwrapCommandResult<T>(
  result: CommandResult<T>,
  fallbackMessage: string,
): T {
  if (result.status === "ok") {
    return result.data;
  }

  throw new Error(result.error.userMessage || fallbackMessage);
}

/** A rejection is an IPC failure with a technical message, so it reads as the fallback. */
export async function callCommand<T>(
  command: Promise<CommandResult<T>>,
  fallbackMessage: string,
): Promise<T> {
  const result = await command.catch(() => {
    throw new Error(fallbackMessage);
  });
  return unwrapCommandResult(result, fallbackMessage);
}
