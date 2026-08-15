import { beforeEach, describe, expect, it, mock } from "bun:test";
import {
  loadTelemetrySettings,
  reportFrontendError,
  saveTelemetryEnabled,
} from "./telemetry";

let enabled = false;

const mockGetSettings = mock(() =>
  Promise.resolve({ enabled, installId: null }),
);
const mockSetEnabled = mock((next: boolean) => {
  enabled = next;
  return Promise.resolve({
    status: "ok" as const,
    data: { enabled: next, installId: "install-id" },
  });
});
const mockCaptureFrontendError = mock(() => Promise.resolve(null));

mock.module("../generated/tauri", () => ({
  commands: {
    telemetryGetSettings: mockGetSettings,
    telemetrySetEnabled: mockSetEnabled,
    telemetryCaptureFrontendError: mockCaptureFrontendError,
  },
}));

describe("telemetry service", () => {
  beforeEach(() => {
    enabled = false;
    mockGetSettings.mockClear();
    mockSetEnabled.mockClear();
    mockCaptureFrontendError.mockClear();
  });

  it("reports telemetry as disabled by default", async () => {
    expect(await loadTelemetrySettings()).toBe(false);
  });

  it("persists the enabled state through the backend command", async () => {
    await saveTelemetryEnabled(true);

    expect(mockSetEnabled).toHaveBeenCalledWith(true);
    expect(await loadTelemetrySettings()).toBe(true);
  });

  it("forwards a frontend error to the backend command", async () => {
    await reportFrontendError({
      source: "render",
      name: "TypeError",
      message: "x is not a function",
      context: "PlanningGrid",
    });

    expect(mockCaptureFrontendError).toHaveBeenCalledWith({
      source: "render",
      name: "TypeError",
      message: "x is not a function",
      context: "PlanningGrid",
    });
  });

  it("never rejects when the backend command fails", async () => {
    mockCaptureFrontendError.mockImplementationOnce(() =>
      Promise.reject(new Error("invoke failed")),
    );

    await reportFrontendError({
      source: "uncaughtError",
      name: "Error",
      message: "boom",
      context: null,
    });
  });
});
