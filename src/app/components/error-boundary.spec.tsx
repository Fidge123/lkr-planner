import { beforeEach, describe, expect, it, mock } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { mockCommands } from "../../test/mock-commands";

const mockCaptureFrontendError = mock(() => Promise.resolve(null));

mockCommands({ telemetryCaptureFrontendError: mockCaptureFrontendError });

const { AppErrorBoundary } = await import("./error-boundary");

describe("AppErrorBoundary", () => {
  beforeEach(() => {
    mockCaptureFrontendError.mockClear();
  });

  it("renders its children while nothing failed", () => {
    const html = renderToStaticMarkup(
      <AppErrorBoundary>
        <p>Wochenplanung</p>
      </AppErrorBoundary>,
    );

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

    const html = renderToStaticMarkup(boundary.render());

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
