import { describe, expect, it } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { WeekTable } from "./week-table";

function renderTable(
  overrides: Partial<Parameters<typeof WeekTable>[0]> = {},
): string {
  return renderToStaticMarkup(
    <WeekTable
      weekDays={[17, 18, 19, 20, 21].map((day) => new Date(2026, 7, day))}
      employees={[]}
      employeeSettings={[]}
      eventsByEmployee={{}}
      errorsByEmployee={{}}
      categoryColors={{}}
      holidays={[]}
      isEmployeeLoading={false}
      onOpenIcalDialog={() => {}}
      onReloadAssignments={() => {}}
      {...overrides}
    />,
  );
}

describe("WeekTable", () => {
  it("labels the header corner with the calendar week of the displayed days", () => {
    expect(renderTable()).toContain("KW 34");
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

  it("follows the displayed week across the year boundary", () => {
    const html = renderTable({
      weekDays: [29, 30, 31].map((day) => new Date(2025, 11, day)),
    });
    expect(html).toContain("KW 1");
  });
});
