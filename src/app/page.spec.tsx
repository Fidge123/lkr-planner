import { beforeAll, describe, expect, it, setSystemTime } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { PlanningGrid, type PlanningGridProjectsState } from "./page";

const defaultState: PlanningGridProjectsState = {
  projects: [],
  isLoading: false,
  errorMessage: null,
  reloadProjects: () => {},
};

describe("planning grid project loading states", () => {
  beforeAll(() => {
    setSystemTime(new Date(2026, 0, 28, 9, 0, 0));
  });

  it("shows initial german loading state when projects are loading", () => {
    const html = renderToStaticMarkup(
      <PlanningGrid
        weekOffset={0}
        projectState={{ ...defaultState, isLoading: true }}
      />,
    );

    expect(html).toContain("Projekte werden geladen...");
  });

  it("shows german error banner with retry action", () => {
    const html = renderToStaticMarkup(
      <PlanningGrid
        weekOffset={0}
        projectState={{
          ...defaultState,
          errorMessage: "Die Daten konnten nicht von Daylite geladen werden.",
        }}
      />,
    );

    expect(html).toContain(
      "Die Daten konnten nicht von Daylite geladen werden.",
    );
    expect(html).toContain("Erneut laden");
  });

  it("renders daylite-backed project names instead of dummy projects", () => {
    const html = renderToStaticMarkup(
      <PlanningGrid
        weekOffset={0}
        projectState={{
          ...defaultState,
          projects: [
            {
              self: "/v1/projects/3001",
              name: "Live Daylite Projekt",
              status: "in_progress",
            },
          ],
        }}
      />,
    );

    expect(html).toContain("Live Daylite Projekt");
    expect(html).not.toContain("Kundenportal");
  });
});
