import { describe, it, mock } from "bun:test";
import { mockCommands } from "../test/mock-commands";
import { reportFrontendError } from "./telemetry";

const mockCaptureFrontendError = mock(() => Promise.resolve(null));

mockCommands({ telemetryCaptureFrontendError: mockCaptureFrontendError });

describe("telemetry service", () => {
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
