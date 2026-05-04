import { describe, expect, it, mock } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import type { CellEvent } from "../types";
import { TimetableCell } from "./timetable-cell";

describe("TimetableCell", () => {
  it("empty cell renders a clickable add affordance", () => {
    const onAddClick = mock(() => {});
    const html = renderToStaticMarkup(
      <TimetableCell
        highlight={false}
        events={[]}
        onAddClick={onAddClick}
        onEventClick={() => {}}
      />,
    );

    expect(html).toContain("Aufgabe hinzufügen");
    expect(html).toContain("<button");
  });

  it("assigned cell renders as clickable with assignment data", () => {
    const onEventClick = mock(() => {});
    const assignment: CellEvent = {
      uid: "uid-1",
      kind: "assignment",
      title: "Bauprojekt Nord",
      color: "bg-primary",
      startTime: "08:00",
      endTime: "16:00",
      href: "/calendars/user/uid-1.ics",
    };

    const html = renderToStaticMarkup(
      <TimetableCell
        highlight={false}
        events={[assignment]}
        onAddClick={() => {}}
        onEventClick={onEventClick}
      />,
    );

    expect(html).toContain("Bauprojekt Nord");
    expect(html).toContain("<button");
  });
});
