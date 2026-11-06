import { beforeEach, describe, expect, it, mock, spyOn } from "bun:test";
import { screen } from "@testing-library/react";
import { mockCommands } from "../../test/mock-commands";
import { renderComponent } from "../../test/render-component";

const mockCaptureFrontendError = mock(() => Promise.resolve(null));

mockCommands({ telemetryCaptureFrontendError: mockCaptureFrontendError });

const { AppErrorBoundary } = await import("./error-boundary");

function PlanningGrid(): never {
  throw new TypeError("x is not a function");
}

describe("AppErrorBoundary", () => {
  beforeEach(() => {
    mockCaptureFrontendError.mockClear();
  });

  it("renders its children while nothing failed", () => {
    renderComponent(
      <AppErrorBoundary>
        <p>Wochenplanung</p>
      </AppErrorBoundary>,
    );

    expect(screen.getByText("Wochenplanung")).toBeDefined();
  });

  it("shows a German fallback and reports the failing component when a child throws", () => {
    const consoleError = spyOn(console, "error").mockImplementation(() => {});

    renderComponent(
      <AppErrorBoundary>
        <PlanningGrid />
      </AppErrorBoundary>,
    );
    consoleError.mockRestore();

    expect(
      screen.getByText("Es ist ein unerwarteter Fehler aufgetreten"),
    ).toBeDefined();
    expect(mockCaptureFrontendError).toHaveBeenCalledWith({
      source: "render",
      name: "TypeError",
      message: "x is not a function",
      context: expect.stringMatching(/^PlanningGrid\b/),
    });
  });
});
