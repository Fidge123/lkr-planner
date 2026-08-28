import { describe, expect, it } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import type { CellEvent } from "../types";
import { TimetableCell } from "./timetable-cell";

function assignment(overrides: Partial<CellEvent> = {}): CellEvent {
  return {
    uid: "uid-1",
    kind: "assignment",
    title: "Bauprojekt Nord",
    color: "bg-base-200",
    startTime: "08:00",
    endTime: "16:00",
    href: "/calendars/user/uid-1.ics",
    projectRef: "/v1/projects/1",
    projectStatus: "in_progress",
    categoryColor: null,
    orderIndex: 0,
    ...overrides,
  };
}

function renderCell(
  overrides: Partial<Parameters<typeof TimetableCell>[0]> = {},
): string {
  return renderToStaticMarkup(
    <TimetableCell
      highlight={false}
      events={[]}
      onAddClick={() => {}}
      onEventClick={() => {}}
      onSuggestionClick={() => {}}
      {...overrides}
    />,
  );
}

const absence = assignment({
  uid: "uid-absence",
  kind: "absence",
  title: "UB",
  color: "bg-(--color-absence-vacation)/50",
  startTime: null,
  endTime: null,
  href: null,
  projectRef: null,
  projectStatus: null,
  orderIndex: null,
});

const bare = assignment({
  uid: "uid-bare",
  kind: "bare",
  title: "Werkstatt",
  startTime: null,
  endTime: null,
  href: null,
  projectRef: null,
  projectStatus: null,
  orderIndex: null,
});

const editLabel = "Einsatz bearbeiten";
const dayliteLabel = "Projekt in Daylite öffnen";

/** Nothing inside the action buttons nests, so a non-greedy match is enough to read them back. */
function buttonContents(html: string): string[] {
  return [...html.matchAll(/<button\b[^>]*>(.*?)<\/button>/gs)].map(
    (match) => match[1],
  );
}

describe("TimetableCell", () => {
  it("empty cell renders a clickable add affordance", () => {
    const html = renderCell();

    expect(html).toContain("Aufgabe hinzufügen");
    expect(html).toContain("<button");
  });

  it("hides the add affordance on a day with an all-day absence", () => {
    const html = renderCell({ events: [absence] });

    expect(html).toContain("UB");
    expect(html).not.toContain("Aufgabe hinzufügen");
  });

  it("hides the add affordance when an all-day absence sits next to a timed one", () => {
    const html = renderCell({
      events: [absence, { ...absence, uid: "uid-2", startTime: "13:00" }],
    });

    expect(html).not.toContain("Aufgabe hinzufügen");
  });

  it("keeps the add affordance when the only absence is a timed one", () => {
    const html = renderCell({
      events: [{ ...absence, startTime: "13:00", endTime: "17:00" }],
    });

    expect(html).toContain("Aufgabe hinzufügen");
  });

  it("assigned cell renders its assignment data", () => {
    const html = renderCell({ events: [assignment()] });

    expect(html).toContain("Bauprojekt Nord");
  });

  it("makes the card itself no control around its times and title", () => {
    const html = renderCell({ events: [assignment()] });

    expect(buttonContents(html)).not.toContainEqual(
      expect.stringContaining("Bauprojekt Nord"),
    );
    expect(buttonContents(html)).not.toContainEqual(
      expect.stringContaining("08:00"),
    );
  });

  it("renders an edit action on an assignment card", () => {
    expect(renderCell({ events: [assignment()] })).toContain(editLabel);
  });

  it("renders the edit action on an assignment with an unresolved project", () => {
    const html = renderCell({
      events: [assignment({ title: "Projekt Süd", projectStatus: null })],
    });

    expect(html).toContain(editLabel);
  });

  it("renders the Daylite action after the edit action on a resolved assignment", () => {
    const html = renderCell({ events: [assignment()] });

    expect(html).toContain(dayliteLabel);
    expect(html.indexOf(editLabel)).toBeLessThan(html.indexOf(dayliteLabel));
  });

  it("stacks the card actions in a column on the card's right edge", () => {
    const html = renderCell({ events: [assignment()] });

    const actionArea = html.slice(
      html.lastIndexOf("<div", html.indexOf(editLabel)),
      html.indexOf(editLabel),
    );

    expect(actionArea).toContain("flex-col");
    expect(actionArea).toContain("shrink-0");
  });

  it("gives the title the card's only flexible column", () => {
    const html = renderCell({ events: [assignment()] });

    expect(html).toContain("grid-cols-[auto_minmax(0,1fr)_auto]");
  });

  it("moves the actions under the times once the column is narrow", () => {
    const html = renderCell({ events: [assignment()] });

    expect(html).toContain("@max-[10rem]:grid-cols-[auto_minmax(0,1fr)]");
    expect(html).toContain("@max-[10rem]:col-start-1");
    expect(html).toContain("@max-[10rem]:row-start-2");
    expect(html).toContain("@max-[10rem]:row-span-2");
  });

  it("renders no Daylite action on an assignment with an unresolved project", () => {
    const html = renderCell({
      events: [assignment({ title: "Projekt Süd", projectStatus: null })],
    });

    expect(html).not.toContain(dayliteLabel);
  });

  it("renders no action area on bare and absence events", () => {
    const html = renderCell({ events: [bare, absence] });

    expect(html).not.toContain(editLabel);
    expect(html).not.toContain(dayliteLabel);
  });

  it("renders no actions on the card being dragged", () => {
    const html = renderCell({
      events: [assignment()],
      draggedUid: "uid-1",
    });

    expect(html).not.toContain(editLabel);
    expect(html).not.toContain(dayliteLabel);
  });

  it("renders a suggestion with reduced opacity and a dashed border", () => {
    const html = renderCell({
      suggestion: {
        date: "2026-05-06",
        projectRef: "/v1/projects/1",
        projectName: "Projekt Vorschlag",
      },
    });

    expect(html).toContain("Projekt Vorschlag");
    expect(html).toContain("opacity-50");
    expect(html).toContain("border-dashed");
    expect(html.indexOf("Projekt Vorschlag")).toBeLessThan(
      html.indexOf("Aufgabe hinzufügen"),
    );
  });

  it("renders no suggestion markup when there is none", () => {
    const html = renderCell();

    expect(html).not.toContain("opacity-50");
    expect(html).not.toContain("border-dashed");
  });

  it("marks assignment cards as draggable", () => {
    expect(renderCell({ events: [assignment()] })).toContain(
      'aria-roledescription="draggable"',
    );
  });

  it("does not make bare or absence events draggable", () => {
    expect(renderCell({ events: [bare, absence] })).not.toContain(
      'aria-roledescription="draggable"',
    );
  });

  it("does not make an assignment with an unresolved project draggable", () => {
    const html = renderCell({
      events: [assignment({ title: "Projekt Süd", projectStatus: null })],
    });

    expect(html).not.toContain('aria-roledescription="draggable"');
  });

  it("marks an unresolved assignment with a red warning icon", () => {
    const html = renderCell({
      events: [assignment({ title: "Projekt Süd", projectStatus: null })],
    });

    expect(html).toContain("lucide-triangle-alert");
    expect(html).toContain("text-error");
  });

  it("sets the note around an unresolved title in italics, the title itself not", () => {
    const html = renderCell({
      events: [assignment({ title: "Projekt Süd", projectStatus: null })],
    });

    expect(html).toContain("<em");
    expect(html).toContain("Beschreibung für ");
    expect(html).toContain(" konnte nicht abgerufen werden");
    expect(html).toContain("</em>Projekt Süd<em");
  });

  it("leaves a resolved assignment title unwrapped and unmarked", () => {
    const html = renderCell({ events: [assignment()] });

    expect(html).not.toContain("<em");
    expect(html).not.toContain("Beschreibung für");
    expect(html).not.toContain("lucide-triangle-alert");
  });

  it("marks a cell with an absence and an assignment as conflicting", () => {
    const html = renderCell({ events: [absence, assignment()] });

    expect(html).toContain("ring-error");
    expect(html).toContain("Abwesenheit und Termin am selben Tag");
    expect(html).toContain("lucide-triangle-alert");
  });

  it("keeps the event colors alongside the conflict indicator", () => {
    const html = renderCell({ events: [absence, assignment()] });

    expect(html).toContain("bg-(--color-absence-vacation)/50");
    expect(html).toContain("bg-base-200");
  });

  it("renders no conflict indicator for an absence without an appointment", () => {
    const html = renderCell({ events: [absence] });

    expect(html).not.toContain("ring-error");
    expect(html).not.toContain("Abwesenheit und Termin am selben Tag");
  });

  it("shows the Daylite category color as a strip, not as the card fill", () => {
    const html = renderCell({
      events: [assignment({ categoryColor: "#8bc34a" })],
    });

    expect(html).toContain("border-left-color:#8bc34a");
    expect(html).not.toContain("background-color:#8bc34a");
    expect(html).toContain("bg-base-200");
  });

  it("passes an unusual but valid CSS color through to the strip untouched", () => {
    expect(
      renderCell({ events: [assignment({ categoryColor: "#8bc34aff" })] }),
    ).toContain("border-left-color:#8bc34aff");
  });

  it("leaves the strip in its default color when there is no category", () => {
    const html = renderCell({ events: [assignment()] });

    expect(html).not.toContain("border-left-color");
    expect(html).toContain("border-base-content/30");
  });

  it("gives bare events no strip", () => {
    const html = renderCell({ events: [bare] });

    expect(html).not.toContain("border-l-4");
    expect(html).toContain("bg-base-200");
  });

  it("renders assignment cards sorted by their order index", () => {
    const html = renderCell({
      events: [
        assignment({ uid: "uid-later", title: "Zweiter", orderIndex: 1 }),
        assignment({ uid: "uid-earlier", title: "Erster", orderIndex: 0 }),
      ],
    });

    expect(html.indexOf("Erster")).toBeLessThan(html.indexOf("Zweiter"));
  });
});

describe("TimetableCell drop preview", () => {
  const first = assignment({
    uid: "uid-first",
    title: "Erster Einsatz",
    orderIndex: 0,
  });
  const second = assignment({
    uid: "uid-second",
    title: "Zweiter Einsatz",
    orderIndex: 1,
  });
  const cell = { employeeRef: "/v1/contacts/1", date: "2026-07-28" };
  const preview = { ...cell, title: "Schwebender Einsatz" };

  const renderPreview = (
    overrides: Partial<Parameters<typeof TimetableCell>[0]> = {},
  ) => renderCell({ ...cell, events: [first, second], ...overrides });

  it("shows the drop preview between the cards it would land between", () => {
    const html = renderPreview({ dropPreview: { ...preview, orderIndex: 1 } });

    expect(html).toContain("Schwebender Einsatz");
    expect(html.indexOf("Erster Einsatz")).toBeLessThan(
      html.indexOf("Schwebender Einsatz"),
    );
    expect(html.indexOf("Schwebender Einsatz")).toBeLessThan(
      html.indexOf("Zweiter Einsatz"),
    );
  });

  it("shows the drop preview above every card for index 0", () => {
    const html = renderPreview({ dropPreview: { ...preview, orderIndex: 0 } });

    expect(html.indexOf("Schwebender Einsatz")).toBeLessThan(
      html.indexOf("Erster Einsatz"),
    );
  });

  it("shows the drop preview below every card when the drop appends", () => {
    const html = renderPreview({ dropPreview: { ...preview, orderIndex: 2 } });

    expect(html.indexOf("Zweiter Einsatz")).toBeLessThan(
      html.indexOf("Schwebender Einsatz"),
    );
  });

  it("skips the dragged card when counting the preview position", () => {
    // The dragged card does not occupy a position in the day it is dropped into, so an
    // index of 1 still lands after the one remaining card.
    const html = renderPreview({
      draggedUid: "uid-first",
      dropPreview: { ...preview, orderIndex: 1 },
    });

    expect(html.indexOf("Zweiter Einsatz")).toBeLessThan(
      html.indexOf("Schwebender Einsatz"),
    );
  });

  it("renders no drop preview when the drag targets another cell", () => {
    const html = renderPreview({
      dropPreview: { ...preview, date: "2026-07-29", orderIndex: 0 },
    });

    expect(html).not.toContain("Schwebender Einsatz");
  });

  it("renders no drop preview when no drag is in flight", () => {
    expect(renderPreview()).not.toContain("drop-preview");
  });
});
