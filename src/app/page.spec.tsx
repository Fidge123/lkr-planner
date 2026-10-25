import { describe, expect, it } from "bun:test";
import type {
  CalendarCellEvent,
  PlanningContactRecord,
} from "../generated/tauri";
import { renderComponent } from "../test/render-component";
import type { HolidaysState } from "./hooks/use-holidays";
import { categoryColorsQuery } from "./hooks/use-project-category-colors";
import {
  type PlanningGridAssignmentState,
  type PlanningGridEmployeesState,
  PlanningGridTable,
  type PlanningGridTableProps,
} from "./page";
import { getWeekDays } from "./util";

const today = "2026-01-28";

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
  getCachedWeek: () => null,
};

const defaultHolidaysState: HolidaysState = {
  holidays: [],
  errorMessage: null,
  reloadHolidays: () => {},
};

function renderGrid({
  showWeekend = false,
  categoryColors = {},
  ...overrides
}: Partial<PlanningGridTableProps> & {
  showWeekend?: boolean;
  categoryColors?: Record<string, string>;
} = {}): string {
  return renderComponent(
    <PlanningGridTable
      weekDays={getWeekDays(today, 0, showWeekend)}
      employeeState={defaultEmployeeState}
      assignmentState={defaultAssignmentState}
      employeeSettings={[]}
      hideNonPlannableEmployees={false}
      holidaysState={defaultHolidaysState}
      onOpenIcalDialog={() => {}}
      {...overrides}
    />,
    [[categoryColorsQuery.queryKey, categoryColors]],
  ).container.innerHTML;
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
    projectPending: false,
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
    expect(html).toContain("border-left-color: #8bc34a");
  });

  it("offers the week picker on the calendar week cell when a jump is passed", () => {
    expect(renderGrid({ onJumpToWeek: () => {} })).toContain(
      "KW 5, Woche auswählen",
    );
    expect(renderGrid()).not.toContain("Woche auswählen");
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
  });
});

describe("planning grid weekend visibility", () => {
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
