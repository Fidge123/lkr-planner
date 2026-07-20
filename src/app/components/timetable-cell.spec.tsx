import { describe, expect, it, mock } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import type { GhostSuggestion } from "../next-day-quick-add";
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
      projectRef: "/v1/projects/1",
      projectStatus: "in_progress",
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

  it("renders a suggestion with reduced opacity and a dashed border", () => {
    const suggestion: GhostSuggestion = {
      date: "2026-05-06",
      projectRef: "/v1/projects/1",
      projectName: "Projekt Vorschlag",
    };

    const html = renderToStaticMarkup(
      <TimetableCell
        highlight={false}
        events={[]}
        suggestion={suggestion}
        onAddClick={() => {}}
        onEventClick={() => {}}
        onSuggestionClick={() => {}}
      />,
    );

    expect(html).toContain("Projekt Vorschlag");
    expect(html).toContain("opacity-50");
    expect(html).toContain("border-dashed");
    expect(html.indexOf("Projekt Vorschlag")).toBeLessThan(
      html.indexOf("Aufgabe hinzufügen"),
    );
  });

  it("renders no suggestion markup when there is none", () => {
    const html = renderToStaticMarkup(
      <TimetableCell
        highlight={false}
        events={[]}
        onAddClick={() => {}}
        onEventClick={() => {}}
      />,
    );

    expect(html).not.toContain("opacity-50");
    expect(html).not.toContain("border-dashed");
  });

  // ── drag-and-drop affordances ──

  const draggableAssignment: CellEvent = {
    uid: "uid-drag",
    kind: "assignment",
    title: "Bauprojekt Süd",
    color: "bg-primary",
    startTime: "08:00",
    endTime: "16:00",
    href: "/calendars/user/uid-drag.ics",
    projectRef: "/v1/projects/7",
    projectStatus: "in_progress",
  };

  it("marks assignment cards as draggable", () => {
    const html = renderToStaticMarkup(
      <TimetableCell
        highlight={false}
        events={[draggableAssignment]}
        onAddClick={() => {}}
        onEventClick={() => {}}
      />,
    );

    expect(html).toContain('aria-roledescription="draggable"');
  });

  it("keeps a plain button for click-to-edit on assignment cards", () => {
    const html = renderToStaticMarkup(
      <TimetableCell
        highlight={false}
        events={[draggableAssignment]}
        onAddClick={() => {}}
        onEventClick={() => {}}
      />,
    );

    expect(html).toContain('type="button"');
    expect(html).toContain("Bauprojekt Süd");
  });

  it("does not make bare or absence events draggable", () => {
    const bare: CellEvent = {
      uid: "uid-bare",
      kind: "bare",
      title: "Werkstatt",
      color: "bg-base-200",
      startTime: null,
      endTime: null,
      href: null,
      projectRef: null,
      projectStatus: null,
    };
    const absence: CellEvent = {
      uid: "uid-abs",
      kind: "absence",
      title: "Urlaub",
      color: "bg-info/30",
      startTime: null,
      endTime: null,
      href: null,
      projectRef: null,
      projectStatus: null,
    };

    const html = renderToStaticMarkup(
      <TimetableCell
        highlight={false}
        events={[bare, absence]}
        onAddClick={() => {}}
        onEventClick={() => {}}
      />,
    );

    expect(html).not.toContain('aria-roledescription="draggable"');
  });

  it("does not make an assignment with an unresolved project draggable", () => {
    const unresolved: CellEvent = {
      ...draggableAssignment,
      uid: "uid-unresolved",
      title: "Beschreibung für Projekt Süd konnte nicht abgerufen werden",
      projectStatus: null,
    };

    const html = renderToStaticMarkup(
      <TimetableCell
        highlight={false}
        events={[unresolved]}
        onAddClick={() => {}}
        onEventClick={() => {}}
      />,
    );

    expect(html).not.toContain('aria-roledescription="draggable"');
    expect(html).toContain("<button");
  });
});
