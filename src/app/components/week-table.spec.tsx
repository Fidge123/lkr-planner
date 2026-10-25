import { describe, expect, it, jest, setSystemTime } from "bun:test";
import { act, screen } from "@testing-library/react";
import type {
  CalendarCellEvent,
  PlanningContactRecord,
} from "../../generated/tauri";
import { renderComponent } from "../../test/render-component";
import { WeekTable } from "./week-table";

function renderTable(
  overrides: Partial<Parameters<typeof WeekTable>[0]> = {},
): string {
  return renderComponent(
    <WeekTable
      weekDays={[17, 18, 19, 20, 21].map((day) => new Date(2026, 7, day))}
      employees={[]}
      employeeSettings={[]}
      eventsByEmployee={{}}
      errorsByEmployee={{}}
      holidays={[]}
      isEmployeeLoading={false}
      onOpenIcalDialog={() => {}}
      {...overrides}
    />,
  ).container.innerHTML;
}

describe("WeekTable", () => {
  it("labels the header corner with the calendar week of the displayed days", () => {
    expect(renderTable()).toContain("KW 34");
  });

  it("moves the today highlight to the next day at midnight", () => {
    const highlightedDays = () =>
      screen
        .getAllByRole("columnheader")
        .filter((header) => header.className.includes("bg-primary"))
        .map((header) => header.querySelector("time")?.dateTime);
    jest.useFakeTimers({ now: new Date(2026, 7, 19, 23, 59, 30) });
    try {
      renderTable();
      expect(highlightedDays()).toEqual(["2026-08-19"]);

      act(() => jest.advanceTimersByTime(60_000));

      expect(highlightedDays()).toEqual(["2026-08-20"]);
    } finally {
      jest.useRealTimers();
    }
  });

  it("keeps the date row fixed at the top of the scrolling grid", () => {
    const html = renderTable();
    const headerCells = html.match(/<th [^>]*>/g) ?? [];
    expect(headerCells.length).toBe(6);
    for (const cell of headerCells) {
      expect(cell).toContain("sticky");
      expect(cell).toContain("top-0");
    }
  });
});

const nord = "/v1/projects/1";
const halle = "/v1/projects/2";

function calendarEvent(
  overrides: Partial<CalendarCellEvent> = {},
): CalendarCellEvent {
  return {
    uid: "uid-1",
    kind: "assignment",
    title: "Bauprojekt Nord",
    projectStatus: "in_progress",
    projectCategory: null,
    date: "2026-08-17",
    startTime: "08:00",
    endTime: "16:00",
    href: "/calendars/user/uid-1.ics",
    projectRef: nord,
    orderIndex: 0,
    projectPending: false,
    ...overrides,
  };
}

function employee(self: string, nickname: string): PlanningContactRecord {
  return { self, nickname, full_name: nickname } as PlanningContactRecord;
}

const kowalski = employee("/v1/contacts/1", "Kowalski");
const meier = employee("/v1/contacts/2", "Meier");

function markedTitles(html: string): string[] {
  return [
    ...html.matchAll(
      /<span class="rounded-sm px-0\.5 bg-\(--color-highlight-marker\)">(.*?)<\/span>/g,
    ),
  ].map((match) => match[1]);
}

function renderWeek(
  eventsByEmployee: Record<string, CalendarCellEvent[]>,
  activeHighlight: string | null,
): string {
  return renderTable({
    employees: [kowalski, meier],
    eventsByEmployee,
    activeHighlight,
  });
}

describe("WeekTable highlighting", () => {
  const spreadWeek = {
    [kowalski.self]: [
      calendarEvent({ uid: "k-mo", date: "2026-08-17" }),
      calendarEvent({
        uid: "k-mo-2",
        date: "2026-08-17",
        title: "Halle 4 Umbau",
        projectRef: halle,
        orderIndex: 1,
      }),
      calendarEvent({ uid: "k-mi", date: "2026-08-19" }),
    ],
    [meier.self]: [
      calendarEvent({ uid: "m-di", date: "2026-08-18" }),
      calendarEvent({
        uid: "m-do",
        date: "2026-08-20",
        title: "Halle 4 Umbau",
        projectRef: halle,
      }),
      calendarEvent({
        uid: "m-fr",
        date: "2026-08-21",
        kind: "bare",
        title: "Bauprojekt Nord",
        projectRef: null,
        projectStatus: null,
      }),
    ],
  };

  it("marks the project across every employee row and every day it sits on", () => {
    const marked = markedTitles(renderWeek(spreadWeek, nord));

    expect(marked).toEqual([
      "Bauprojekt Nord",
      "Bauprojekt Nord",
      "Bauprojekt Nord",
    ]);
  });

  it("marks nothing and changes nothing when every matching event is gone", () => {
    const withoutNord = {
      [kowalski.self]: [
        calendarEvent({
          uid: "k-mo-2",
          title: "Halle 4 Umbau",
          projectRef: halle,
        }),
      ],
    };

    expect(markedTitles(renderWeek(withoutNord, nord))).toEqual([]);
    expect(renderWeek(withoutNord, nord)).toBe(renderWeek(withoutNord, null));
  });

  it("marks a card sitting in the current day's tinted column", () => {
    setSystemTime(new Date(2026, 7, 19, 9));
    try {
      const html = renderWeek(spreadWeek, nord);
      const todayCell = [
        ...html.matchAll(/<td[^>]*bg-primary\/10[^>]*>(.*?)<\/td>/gs),
      ];

      expect(todayCell.length).toBe(2);
      expect(markedTitles(todayCell[0][1])).toEqual(["Bauprojekt Nord"]);
    } finally {
      setSystemTime();
    }
  });
});
