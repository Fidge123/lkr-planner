import type { AppError } from "../generated/tauri";

type CommandResult<T> =
  | { status: "ok"; data: T }
  | { status: "error"; error: AppError };

export function appError(userMessage: string): AppError {
  return {
    code: "COMMAND_FAILED",
    httpStatus: null,
    userMessage,
    technicalMessage: userMessage,
  };
}

export function commandOk<T>(data: T): Promise<CommandResult<T>> {
  return Promise.resolve({ status: "ok", data });
}

export function commandFailed<T>(
  userMessage: string,
): Promise<CommandResult<T>> {
  return Promise.resolve({ status: "error", error: appError(userMessage) });
}
