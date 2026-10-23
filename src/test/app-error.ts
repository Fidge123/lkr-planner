import type { AppError } from "../generated/tauri";

export function appError(userMessage: string): AppError {
  return {
    code: "COMMAND_FAILED",
    httpStatus: null,
    userMessage,
    technicalMessage: userMessage,
  };
}
