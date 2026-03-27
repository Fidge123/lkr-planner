import { beforeAll, describe, expect, it, setSystemTime } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import {
  type PlanningGridEmployeesState,
  type PlanningGridProjectsState,
  PlanningGridTable,
} from "./page";

const defaultState: PlanningGridProjectsState = {
  projects: [],
  isLoading: false,
  errorMessage: null,
  reloadProjects: () => {},
};

const defaultEmployeeState: PlanningGridEmployeesState = {
  employees: [],
  isLoading: false,
  errorMessage: null,
  reloadEmployees: () => {},
};

const defaultIcalProps = {
  employeeSettings: [] as import("../generated/tauri").EmployeeSetting[],
  onOpenIcalDialog: () => {},
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
        employeeState={{ ...defaultEmployeeState }}
        {...defaultIcalProps}
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
        employeeState={{ ...defaultEmployeeState }}
        {...defaultIcalProps}
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
              keywords: [],
            },
          ],
        }}
        employeeState={{ ...defaultEmployeeState }}
        {...defaultIcalProps}
      />,
    );

    expect(html).toContain("Live Daylite Projekt");
    expect(html).not.toContain("Kundenportal");
  });

  it("shows empty state when no projects are loaded", () => {
    const html = renderToStaticMarkup(
      <PlanningGridTable
        weekOffset={0}
        projectState={{ ...defaultState }}
        employeeState={{ ...defaultEmployeeState }}
        {...defaultIcalProps}
      />,
    );

    const tableIndex = html.indexOf("</table>");
    const sectionLabelIndex = html.indexOf("Geladene Projekte");
    const emptyStateIndex = html.indexOf("Keine Projekte gefunden");

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
              keywords: [],
              due: "2026-02-20T00:00:00.000Z",
            },
          ],
        }}
        employeeState={{ ...defaultEmployeeState }}
        {...defaultIcalProps}
      />,
    );

    expect(html).toContain("In Arbeit");
    expect(html).toContain("20.02.2026");
  });

  it("does not crash when a project record is missing self", () => {
    const html = renderToStaticMarkup(
      <PlanningGridTable
        weekOffset={0}
        projectState={{
          ...defaultState,
          projects: [
            {
              name: "Projekt Ohne Self",
              status: "in_progress",
            } as unknown as PlanningGridProjectsState["projects"][number],
          ],
        }}
        employeeState={{ ...defaultEmployeeState }}
        {...defaultIcalProps}
      />,
    );

    expect(html).toContain("Projekt Ohne Self");
    expect(html).toContain("In Arbeit");
  });

  it("renders daylite-backed employee names instead of dummy employee names", () => {
    const html = renderToStaticMarkup(
      <PlanningGridTable
        weekOffset={0}
        projectState={{ ...defaultState }}
        employeeState={{
          ...defaultEmployeeState,
          employees: [
            {
              self: "/v1/contacts/9001",
              full_name: "Monteur Aus Daylite",
              category: "Monteur",
              urls: [],
            },
          ],
        }}
        {...defaultIcalProps}
      />,
    );

    expect(html).toContain("Monteur Aus Daylite");
    expect(html).not.toContain("Anna Schmidt");
  });
});
