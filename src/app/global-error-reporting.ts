import { reportFrontendError } from "../services/telemetry";

export function installGlobalErrorReporting(target: EventTarget): () => void {
  const onError = (event: Event) => {
    report("uncaughtError", (event as Event & { error: unknown }).error);
  };
  const onRejection = (event: Event) => {
    report("unhandledRejection", (event as Event & { reason: unknown }).reason);
  };

  target.addEventListener("error", onError);
  target.addEventListener("unhandledrejection", onRejection);

  return () => {
    target.removeEventListener("error", onError);
    target.removeEventListener("unhandledrejection", onRejection);
  };
}

function report(
  source: "uncaughtError" | "unhandledRejection",
  cause: unknown,
): void {
  void reportFrontendError({
    source,
    name: cause instanceof Error ? cause.name : "UnknownError",
    message: cause instanceof Error ? cause.message : String(cause),
    context: null,
  });
}
