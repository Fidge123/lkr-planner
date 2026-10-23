import { beforeEach, describe, expect, it, mock } from "bun:test";
import { mockCommands } from "../../test/mock-commands";
import { renderComponent } from "../../test/render-component";

const mockCaptureFrontendError = mock(() => Promise.resolve(null));

mockCommands({ telemetryCaptureFrontendError: mockCaptureFrontendError });

const { AppErrorBoundary } = await import("./error-boundary");

describe("AppErrorBoundary", () => {
  beforeEach(() => {
    mockCaptureFrontendError.mockClear();
  });

  it("renders its children while nothing failed", () => {
    const html = renderComponent(
      <AppErrorBoundary>
        <p>Wochenplanung</p>
      </AppErrorBoundary>,
    ).container.innerHTML;

    expect(html).toContain("Wochenplanung");
  });

  it("switches to the error state when a child throws", () => {
    expect(AppErrorBoundary.getDerivedStateFromError()).toEqual({
      hasError: true,
    });
  });

  it("shows a German fallback message instead of a blank screen", () => {
    const boundary = new AppErrorBoundary({ children: null });
    boundary.state = { hasError: true };

    const html = renderComponent(boundary.render()).container.innerHTML;

    expect(html).toContain("Es ist ein unerwarteter Fehler aufgetreten");
  });

  it("reports the render error through the telemetry command", () => {
    const boundary = new AppErrorBoundary({ children: null });

    boundary.componentDidCatch(new TypeError("x is not a function"), {
      componentStack: "\n    at PlanningGrid\n    at App",
    });

    expect(mockCaptureFrontendError).toHaveBeenCalledWith({
      source: "render",
      name: "TypeError",
      message: "x is not a function",
      context: "PlanningGrid",
    });
  });
});
