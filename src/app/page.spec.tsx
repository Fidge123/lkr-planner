import { beforeAll, describe, expect, it, setSystemTime } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { type PlanningGridProjectsState, PlanningGridTable } from "./page";

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
      <PlanningGridTable
        weekOffset={0}
        projectState={{ ...defaultState, isLoading: true }}
      />,
    );

    expect(html).toContain("Geladene Projekte");
    expect(html).toContain("Projekte werden geladen...");
  });

  it("shows german error banner with retry action", () => {
    const html = renderToStaticMarkup(
      <PlanningGridTable
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
      <PlanningGridTable
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

  it("shows empty state for an empty standard filter result", () => {
    const html = renderToStaticMarkup(
      <PlanningGridTable weekOffset={0} projectState={{ ...defaultState }} />,
    );

    const tableIndex = html.indexOf("</table>");
    const sectionLabelIndex = html.indexOf("Geladene Projekte");
    const emptyStateIndex = html.indexOf(
      "Keine Projekte im Standard-Filter gefunden",
    );

    expect(tableIndex).toBeGreaterThan(-1);
    expect(sectionLabelIndex).toBeGreaterThan(tableIndex);
    expect(emptyStateIndex).toBeGreaterThan(sectionLabelIndex);
  });

  it("renders project status and due date in loaded projects section", () => {
    const html = renderToStaticMarkup(
      <PlanningGridTable
        weekOffset={0}
        projectState={{
          ...defaultState,
          projects: [
            {
              self: "/v1/projects/3001",
              name: "Live Daylite Projekt",
              status: "in_progress",
              due: "2026-02-20T00:00:00.000Z",
            },
          ],
        }}
      />,
    );

    expect(html).toContain("In Arbeit");
    expect(html).toContain("20.02.2026");
  });
});
