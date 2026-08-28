import { beforeEach, describe, expect, it, mock } from "bun:test";

const mockReportFrontendError = mock(() => Promise.resolve());
const realTelemetry = await import("../services/telemetry");

mock.module("../services/telemetry", () => ({
  ...realTelemetry,
  reportFrontendError: mockReportFrontendError,
}));

const { installGlobalErrorReporting } = await import(
  "./global-error-reporting"
);

describe("global error reporting", () => {
  beforeEach(() => {
    mockReportFrontendError.mockClear();
  });

  it("reports an uncaught error with its name and message", () => {
    const target = new EventTarget();
    installGlobalErrorReporting(target);

    const event = new Event("error") as Event & { error: unknown };
    event.error = new RangeError("out of range");
    target.dispatchEvent(event);

    expect(mockReportFrontendError).toHaveBeenCalledWith({
      source: "uncaughtError",
      name: "RangeError",
      message: "out of range",
      context: null,
    });
  });

  it("reports an unhandled promise rejection", () => {
    const target = new EventTarget();
    installGlobalErrorReporting(target);

    const event = new Event("unhandledrejection") as Event & {
      reason: unknown;
    };
    event.reason = new Error("load failed");
    target.dispatchEvent(event);

    expect(mockReportFrontendError).toHaveBeenCalledWith({
      source: "unhandledRejection",
      name: "Error",
      message: "load failed",
      context: null,
    });
  });

  it("reports a rejection whose reason is not an Error", () => {
    const target = new EventTarget();
    installGlobalErrorReporting(target);

    const event = new Event("unhandledrejection") as Event & {
      reason: unknown;
    };
    event.reason = "kaputt";
    target.dispatchEvent(event);

    expect(mockReportFrontendError).toHaveBeenCalledWith({
      source: "unhandledRejection",
      name: "UnknownError",
      message: "kaputt",
      context: null,
    });
  });

  it("stops reporting once the listeners are removed", () => {
    const target = new EventTarget();
    const uninstall = installGlobalErrorReporting(target);
    uninstall();

    const event = new Event("error") as Event & { error: unknown };
    event.error = new Error("boom");
    target.dispatchEvent(event);

    expect(mockReportFrontendError).not.toHaveBeenCalled();
  });
});
