import { beforeAll, describe, expect, it, setSystemTime } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import type { CalendarCellEvent } from "../generated/tauri";
import type { HolidaysState } from "./hooks/use-holidays";
import {
  PlanningGrid,
  type PlanningGridAssignmentState,
  type PlanningGridEmployeesState,
  type PlanningGridProjectsState,
  PlanningGridTable,
} from "./page";
import { getWeekDays } from "./util";

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

const defaultAssignmentState: PlanningGridAssignmentState = {
  eventsByEmployee: {},
  errorsByEmployee: {},
  isLoading: false,
  errorMessage: null,
  reloadAssignments: () => {},
};

const defaultHolidaysState: HolidaysState = {
  holidays: [],
  isLoading: false,
  errorMessage: null,
  reloadHolidays: () => {},
};

const defaultIcalProps = {
  employeeSettings: [] as import("../generated/tauri").EmployeeSetting[],
  hideNonPlannableEmployees: false,
  holidaysState: defaultHolidaysState,
  onOpenIcalDialog: () => {},
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
        employeeState={{ ...defaultEmployeeState }}
        assignmentState={{ ...defaultAssignmentState }}
        {...defaultIcalProps}
      />,
    );

    expect(html).toContain("Geladene Projekte");
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
        employeeState={{ ...defaultEmployeeState }}
        assignmentState={{ ...defaultAssignmentState }}
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
      <PlanningGrid
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
        assignmentState={{ ...defaultAssignmentState }}
        {...defaultIcalProps}
      />,
    );

    expect(html).toContain("Live Daylite Projekt");
    expect(html).not.toContain("Kundenportal");
  });

  it("shows empty state when no projects are loaded", () => {
    const html = renderToStaticMarkup(
      <PlanningGrid
        weekOffset={0}
        projectState={{ ...defaultState }}
        employeeState={{ ...defaultEmployeeState }}
        assignmentState={{ ...defaultAssignmentState }}
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
      <PlanningGrid
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
        assignmentState={{ ...defaultAssignmentState }}
        {...defaultIcalProps}
      />,
    );

    expect(html).toContain("In Arbeit");
    expect(html).toContain("20.02.2026");
  });

  it("does not crash when a project record is missing self", () => {
    const html = renderToStaticMarkup(
      <PlanningGrid
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
        assignmentState={{ ...defaultAssignmentState }}
        {...defaultIcalProps}
      />,
    );

    expect(html).toContain("Projekt Ohne Self");
    expect(html).toContain("In Arbeit");
  });

  it("renders daylite-backed employee names instead of dummy employee names", () => {
    const html = renderToStaticMarkup(
      <PlanningGrid
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
        assignmentState={{ ...defaultAssignmentState }}
        {...defaultIcalProps}
      />,
    );

    expect(html).toContain("Monteur Aus Daylite");
    expect(html).not.toContain("Anna Schmidt");
  });

  it("hides employees without a calendar and test employees when the toggle is enabled", () => {
    const html = renderToStaticMarkup(
      <PlanningGrid
        weekOffset={0}
        projectState={{ ...defaultState }}
        employeeState={{
          ...defaultEmployeeState,
          employees: [
            {
              self: "/v1/contacts/1",
              full_name: "Mit Kalender",
              category: "Monteur",
              urls: [],
            },
            {
              self: "/v1/contacts/2",
              full_name: "Ohne Kalender",
              category: "Monteur",
              urls: [],
            },
            {
              self: "/v1/contacts/3",
              full_name: "Test Mitarbeiter",
              category: "Test",
              urls: [],
            },
          ],
        }}
        assignmentState={{ ...defaultAssignmentState }}
        {...defaultIcalProps}
        hideNonPlannableEmployees={true}
        employeeSettings={[
          {
            dayliteContactReference: "/v1/contacts/1",
            zepPrimaryCalendar: "https://app.zep.de/caldav/admin/eins/",
          },
          {
            dayliteContactReference: "/v1/contacts/3",
            zepPrimaryCalendar: "https://app.zep.de/caldav/admin/drei/",
          },
        ]}
      />,
    );

    expect(html).toContain("Mit Kalender");
    expect(html).not.toContain("Ohne Kalender");
    expect(html).not.toContain("Test Mitarbeiter");
  });
});

describe("planning grid assignment states", () => {
  beforeAll(() => {
    setSystemTime(new Date(2026, 0, 28, 9, 0, 0));
  });

  it("shows german loading state when assignments are loading", () => {
    const html = renderToStaticMarkup(
      <PlanningGrid
        weekOffset={0}
        projectState={{ ...defaultState }}
        employeeState={{ ...defaultEmployeeState }}
        assignmentState={{ ...defaultAssignmentState, isLoading: true }}
        {...defaultIcalProps}
      />,
    );

    expect(html).toContain("Einsätze werden geladen...");
  });

  it("shows german error banner with retry when assignment fetch fails", () => {
    const html = renderToStaticMarkup(
      <PlanningGrid
        weekOffset={0}
        projectState={{ ...defaultState }}
        employeeState={{ ...defaultEmployeeState }}
        assignmentState={{
          ...defaultAssignmentState,
          errorMessage: "Die Einsätze konnten nicht geladen werden.",
        }}
        {...defaultIcalProps}
      />,
    );

    expect(html).toContain("Die Einsätze konnten nicht geladen werden.");
    expect(html).toContain("Erneut laden");
  });

  it("renders lkr-planner assignment event in cell with project color", () => {
    const employee = {
      self: "/v1/contacts/9001",
      full_name: "Monteur Aus Daylite",
      category: "Monteur",
      urls: [],
    };
    const assignmentEvent: CalendarCellEvent = {
      uid: "event-uid-1",
      kind: "assignment",
      title: "Projekt Nord",
      projectStatus: "in_progress",
      projectRef: "/v1/projects/1",
      date: "2026-01-26",
      startTime: null,
      endTime: null,
      href: null,
    };

    const html = renderToStaticMarkup(
      <PlanningGrid
        weekOffset={0}
        projectState={{ ...defaultState }}
        employeeState={{ ...defaultEmployeeState, employees: [employee] }}
        assignmentState={{
          ...defaultAssignmentState,
          eventsByEmployee: {
            "/v1/contacts/9001": [assignmentEvent],
          },
        }}
        {...defaultIcalProps}
      />,
    );

    expect(html).toContain("Projekt Nord");
    expect(html).toContain("bg-secondary");
  });

  it("renders bare event in cell with neutral style and no edit affordance", () => {
    const employee = {
      self: "/v1/contacts/9001",
      full_name: "Monteur Aus Daylite",
      category: "Monteur",
      urls: [],
    };
    const bareEvent: CalendarCellEvent = {
      uid: "bare-uid-1",
      kind: "bare",
      title: "Auto Werkstatt",
      projectStatus: null,
      projectRef: null,
      date: "2026-01-26",
      startTime: null,
      endTime: null,
      href: null,
    };

    const html = renderToStaticMarkup(
      <PlanningGrid
        weekOffset={0}
        projectState={{ ...defaultState }}
        employeeState={{ ...defaultEmployeeState, employees: [employee] }}
        assignmentState={{
          ...defaultAssignmentState,
          eventsByEmployee: {
            "/v1/contacts/9001": [bareEvent],
          },
        }}
        {...defaultIcalProps}
      />,
    );

    expect(html).toContain("Auto Werkstatt");
    expect(html).toContain("bg-base-200");
    expect(html).not.toContain("bg-secondary");
  });

  it("renders empty cells when no events exist for the week", () => {
    const employee = {
      self: "/v1/contacts/9001",
      full_name: "Monteur Aus Daylite",
      category: "Monteur",
      urls: [],
    };

    const html = renderToStaticMarkup(
      <PlanningGrid
        weekOffset={0}
        projectState={{ ...defaultState }}
        employeeState={{ ...defaultEmployeeState, employees: [employee] }}
        assignmentState={{ ...defaultAssignmentState, eventsByEmployee: {} }}
        {...defaultIcalProps}
      />,
    );

    expect(html).toContain("Monteur Aus Daylite");
    expect(html).not.toContain("bg-secondary");
  });

  it("renders dates from the next week when weekOffset is 1", () => {
    const html = renderToStaticMarkup(
      <PlanningGrid
        weekOffset={1}
        projectState={{ ...defaultState }}
        employeeState={{ ...defaultEmployeeState }}
        assignmentState={{ ...defaultAssignmentState }}
        {...defaultIcalProps}
      />,
    );

    expect(html).toContain("02.02");
  });

  it("renders per-employee calendar error inline in the row", () => {
    const employee = {
      self: "/v1/contacts/9001",
      full_name: "Monteur Aus Daylite",
      category: "Monteur",
      urls: [],
    };

    const html = renderToStaticMarkup(
      <PlanningGrid
        weekOffset={0}
        projectState={{ ...defaultState }}
        employeeState={{ ...defaultEmployeeState, employees: [employee] }}
        assignmentState={{
          ...defaultAssignmentState,
          errorsByEmployee: {
            "/v1/contacts/9001": "CalDAV server unreachable",
          },
        }}
        {...defaultIcalProps}
      />,
    );

    expect(html).toContain("Kalender nicht verfügbar");
    expect(html).toContain("CalDAV server unreachable");
    expect(html).toContain("Erneut laden");
    expect(html).not.toContain("bg-secondary");
  });
});

describe("planning grid drag-and-drop wiring", () => {
  beforeAll(() => {
    setSystemTime(new Date(2026, 0, 28, 9, 0, 0));
  });

  const employee = {
    self: "/v1/contacts/9001",
    full_name: "Monteur Aus Daylite",
    category: "Monteur",
    urls: [],
  };
  const assignmentEvent: CalendarCellEvent = {
    uid: "event-uid-1",
    kind: "assignment",
    title: "Projekt Nord",
    projectStatus: "in_progress",
    projectRef: "/v1/projects/1",
    date: "2026-01-26",
    startTime: "08:00",
    endTime: "16:00",
    href: "/calendars/user/event-uid-1.ics",
  };

  const renderGrid = () =>
    renderToStaticMarkup(
      <PlanningGridTable
        weekDays={getWeekDays(0)}
        projectState={{ ...defaultState }}
        employeeState={{ ...defaultEmployeeState, employees: [employee] }}
        assignmentState={{
          ...defaultAssignmentState,
          eventsByEmployee: { "/v1/contacts/9001": [assignmentEvent] },
        }}
        {...defaultIcalProps}
        onNavigateWeek={() => {}}
      />,
    );

  it("renders assignment cards as draggable inside the drag context", () => {
    const html = renderGrid();

    expect(html).toContain("Projekt Nord");
    expect(html).toContain('aria-roledescription="draggable"');
  });

  it("shows no drop error and no reconciliation dialog before any drag", () => {
    const html = renderGrid();

    expect(html).not.toContain("Einsatz doppelt vorhanden");
    expect(html).not.toContain("alert-error");
  });
});

describe("planning grid weekend visibility", () => {
  beforeAll(() => {
    setSystemTime(new Date(2026, 0, 28, 9, 0, 0));
  });

  const countDayColumns = (html: string) => (html.match(/<time/g) ?? []).length;

  it("renders 5 day columns by default (weekend hidden)", () => {
    const html = renderToStaticMarkup(
      <PlanningGrid
        weekOffset={0}
        projectState={{ ...defaultState }}
        employeeState={{ ...defaultEmployeeState }}
        assignmentState={{ ...defaultAssignmentState }}
        employeeSettings={[]}
        onOpenIcalDialog={() => {}}
      />,
    );

    expect(countDayColumns(html)).toBe(5);
  });

  it("renders 7 day columns when showWeekend is on", () => {
    const html = renderToStaticMarkup(
      <PlanningGrid
        weekOffset={0}
        showWeekend
        projectState={{ ...defaultState }}
        employeeState={{ ...defaultEmployeeState }}
        assignmentState={{ ...defaultAssignmentState }}
        employeeSettings={[]}
        onOpenIcalDialog={() => {}}
      />,
    );

    expect(countDayColumns(html)).toBe(7);
  });

  it("displays a holiday that falls on a weekend day when showWeekend is on", () => {
    const html = renderToStaticMarkup(
      <PlanningGrid
        weekOffset={0}
        showWeekend
        projectState={{ ...defaultState }}
        employeeState={{ ...defaultEmployeeState }}
        assignmentState={{ ...defaultAssignmentState }}
        employeeSettings={[]}
        hideNonPlannableEmployees={false}
        holidaysState={{
          ...defaultHolidaysState,
          holidays: [{ date: "2026-01-31", name: "Test-Samstagsfeiertag" }],
        }}
        onOpenIcalDialog={() => {}}
      />,
    );

    expect(html).toContain("Test-Samstagsfeiertag");
  });
});
