import { beforeAll, describe, expect, it, setSystemTime } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import type {
  CalendarCellEvent,
  PlanningContactRecord,
} from "../generated/tauri";
import type { HolidaysState } from "./hooks/use-holidays";
import {
  PlanningGrid,
  type PlanningGridAssignmentState,
  type PlanningGridEmployeesState,
  PlanningGridTable,
} from "./page";
import { getWeekDays } from "./util";

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
  loadedWeekStart: null,
  reloadAssignments: () => {},
  invalidateWeeksContaining: () => {},
  getCachedWeek: () => null,
};

const defaultHolidaysState: HolidaysState = {
  holidays: [],
  isLoading: false,
  errorMessage: null,
  reloadHolidays: () => {},
};

function renderGrid(
  overrides: Partial<Parameters<typeof PlanningGrid>[0]> = {},
): string {
  return renderToStaticMarkup(
    <PlanningGrid
      weekOffset={0}
      employeeState={defaultEmployeeState}
      assignmentState={defaultAssignmentState}
      employeeSettings={[]}
      hideNonPlannableEmployees={false}
      holidaysState={defaultHolidaysState}
      onOpenIcalDialog={() => {}}
      {...overrides}
    />,
  );
}

const employee: PlanningContactRecord = {
  self: "/v1/contacts/9001",
  full_name: "Monteur Aus Daylite",
  nickname: null,
  category: "Monteur",
  urls: [],
};

function cellEvent(overrides: Partial<CalendarCellEvent>): CalendarCellEvent {
  return {
    uid: "event-uid-1",
    kind: "assignment",
    title: "Projekt Nord",
    projectStatus: "in_progress",
    projectCategory: null,
    projectRef: "/v1/projects/1",
    date: "2026-01-26",
    startTime: null,
    endTime: null,
    href: null,
    orderIndex: null,
    ...overrides,
  };
}

/** One employee whose row carries the given events. */
function withEvents(events: CalendarCellEvent[]) {
  return {
    employeeState: { ...defaultEmployeeState, employees: [employee] },
    assignmentState: {
      ...defaultAssignmentState,
      eventsByEmployee: { [employee.self]: events },
    },
  };
}

describe("planning grid employee states", () => {
  beforeAll(() => {
    setSystemTime(new Date(2026, 0, 28, 9, 0, 0));
  });

  it("renders no loaded projects section below the grid", () => {
    const html = renderGrid();

    expect(html).not.toContain("Geladene Projekte");
    expect(html).not.toContain("Keine Projekte gefunden");
  });

  it("renders daylite-backed employee names instead of dummy employee names", () => {
    const html = renderGrid({
      employeeState: { ...defaultEmployeeState, employees: [employee] },
    });

    expect(html).toContain("Monteur Aus Daylite");
    expect(html).not.toContain("Anna Schmidt");
  });

  it("hides employees without a calendar and test employees when the toggle is enabled", () => {
    const html = renderGrid({
      employeeState: {
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
      },
      hideNonPlannableEmployees: true,
      employeeSettings: [
        {
          dayliteContactReference: "/v1/contacts/1",
          zepPrimaryCalendar: "https://app.zep.de/caldav/admin/eins/",
        },
        {
          dayliteContactReference: "/v1/contacts/3",
          zepPrimaryCalendar: "https://app.zep.de/caldav/admin/drei/",
        },
      ],
    });

    expect(html).toContain("Mit Kalender");
    expect(html).not.toContain("Ohne Kalender");
    expect(html).not.toContain("Test Mitarbeiter");
  });
});

describe("planning grid assignment states", () => {
  beforeAll(() => {
    setSystemTime(new Date(2026, 0, 28, 9, 0, 0));
  });

  it("shows no loading text above the grid while assignments load", () => {
    const html = renderGrid({
      assignmentState: { ...defaultAssignmentState, isLoading: true },
    });

    expect(html).not.toContain("Einsätze werden geladen...");
    expect(html).not.toContain("werden geladen");
  });

  it("shows german error banner with retry when assignment fetch fails", () => {
    const html = renderGrid({
      assignmentState: {
        ...defaultAssignmentState,
        errorMessage: "Die Einsätze konnten nicht geladen werden.",
      },
    });

    expect(html).toContain("Die Einsätze konnten nicht geladen werden.");
    expect(html).toContain("Erneut laden");
  });

  it("renders lkr-planner assignment event in cell with its category color", () => {
    const html = renderGrid({
      ...withEvents([cellEvent({ projectCategory: "Bau" })]),
      categoryColors: { Bau: "#8bc34a" },
    });

    expect(html).toContain("Projekt Nord");
    expect(html).toContain("border-left-color:#8bc34a");
  });

  it("renders bare event in cell with neutral style and no edit affordance", () => {
    const html = renderGrid(
      withEvents([
        cellEvent({
          uid: "bare-uid-1",
          kind: "bare",
          title: "Auto Werkstatt",
          projectStatus: null,
          projectRef: null,
        }),
      ]),
    );

    expect(html).toContain("Auto Werkstatt");
    expect(html).toContain("bg-base-200");
    expect(html).not.toContain("bg-secondary");
  });

  it("renders empty cells when no events exist for the week", () => {
    const html = renderGrid(withEvents([]));

    expect(html).toContain("Monteur Aus Daylite");
    expect(html).not.toContain("bg-secondary");
  });

  it("renders dates from the next week when weekOffset is 1", () => {
    expect(renderGrid({ weekOffset: 1 })).toContain("02.02");
  });

  it("renders per-employee calendar error inline in the row", () => {
    const html = renderGrid({
      employeeState: { ...defaultEmployeeState, employees: [employee] },
      assignmentState: {
        ...defaultAssignmentState,
        errorsByEmployee: { [employee.self]: "CalDAV server unreachable" },
      },
    });

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

  const renderGridTable = () =>
    renderToStaticMarkup(
      <PlanningGridTable
        weekDays={getWeekDays(0)}
        employeeSettings={[]}
        hideNonPlannableEmployees={false}
        holidaysState={defaultHolidaysState}
        categoryColors={{}}
        onOpenIcalDialog={() => {}}
        onNavigateWeek={() => {}}
        {...withEvents([
          cellEvent({ startTime: "08:00", endTime: "16:00", href: "/cal.ics" }),
        ])}
      />,
    );

  it("renders assignment cards as draggable inside the drag context", () => {
    const html = renderGridTable();

    expect(html).toContain("Projekt Nord");
    expect(html).toContain('aria-roledescription="draggable"');
  });

  it("shows no drop error and no reconciliation dialog before any drag", () => {
    const html = renderGridTable();

    expect(html).not.toContain("Einsatz doppelt vorhanden");
    expect(html).not.toContain("alert-error");
  });
});

describe("planning grid weekend visibility", () => {
  beforeAll(() => {
    setSystemTime(new Date(2026, 0, 28, 9, 0, 0));
  });

  const countDayColumns = (html: string) => (html.match(/<time/g) ?? []).length;

  it("gives each day column a machine-readable date matching its visible date", () => {
    const columns = [
      ...renderGrid().matchAll(/<time dateTime="([^"]+)"[^>]*>([^<]+)</gi),
    ];

    expect(columns).toHaveLength(5);
    for (const [, machineDate, visibleText] of columns) {
      const [, month, day] = machineDate.split("-");
      expect(visibleText).toContain(`${day}.${month}.`);
    }
  });

  it("renders 5 day columns by default (weekend hidden)", () => {
    expect(countDayColumns(renderGrid())).toBe(5);
  });

  it("renders 7 day columns when showWeekend is on", () => {
    expect(countDayColumns(renderGrid({ showWeekend: true }))).toBe(7);
  });

  it("displays a holiday that falls on a weekend day when showWeekend is on", () => {
    const html = renderGrid({
      showWeekend: true,
      holidaysState: {
        ...defaultHolidaysState,
        holidays: [{ date: "2026-01-31", name: "Test-Samstagsfeiertag" }],
      },
    });

    expect(html).toContain("Test-Samstagsfeiertag");
  });
});
