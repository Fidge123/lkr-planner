type CommandResult<T, E> =
  | { status: "ok"; data: T }
  | { status: "error"; error: E };

export function unwrapCommandResult<T, E>(
  result: CommandResult<T, E>,
  fallbackMessage: string,
): T {
  if (result.status === "ok") {
    return result.data;
  }

  throw new Error(readErrorMessage(result.error, fallbackMessage));
}

function readErrorMessage(error: unknown, fallbackMessage: string): string {
  if (typeof error === "string") {
    return error;
  }

  if (
    error &&
    typeof error === "object" &&
    "userMessage" in error &&
    typeof (error as { userMessage: unknown }).userMessage === "string" &&
    (error as { userMessage: string }).userMessage.length > 0
  ) {
    return (error as { userMessage: string }).userMessage;
  }

  return fallbackMessage;
}
