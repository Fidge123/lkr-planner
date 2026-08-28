import { beforeEach, describe, expect, it, mock } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";

const mockReportFrontendError = mock(() => Promise.resolve());
const realTelemetry = await import("../../services/telemetry");

mock.module("../../services/telemetry", () => ({
  ...realTelemetry,
  reportFrontendError: mockReportFrontendError,
}));

const { AppErrorBoundary } = await import("./error-boundary");

describe("AppErrorBoundary", () => {
  beforeEach(() => {
    mockReportFrontendError.mockClear();
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

  it("reports the render error through the telemetry service", () => {
    const boundary = new AppErrorBoundary({ children: null });

    boundary.componentDidCatch(new TypeError("x is not a function"), {
      componentStack: "\n    at PlanningGrid\n    at App",
    });

    expect(mockReportFrontendError).toHaveBeenCalledWith({
      source: "render",
      name: "TypeError",
      message: "x is not a function",
      context: "PlanningGrid",
    });
  });
});
