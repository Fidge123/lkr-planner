import { describe, expect, it, mock } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import type {
  CalendarCellEvent,
  DayliteProjectSummary,
} from "../../generated/tauri";
import {
  AssignmentModal,
  nextHighlightIndex,
  ProjectResultList,
  resolveEscapeAction,
} from "./assignment-modal";

mock.module("../../generated/tauri", () => ({
  commands: {
    createAssignment: mock(() => Promise.resolve({ status: "ok", data: "" })),
    updateAssignment: mock(() => Promise.resolve({ status: "ok", data: null })),
    deleteAssignment: mock(() => Promise.resolve({ status: "ok", data: null })),
  },
}));

const baseProps = {
  employeeReference: "ref-123",
  date: "2026-05-06",
  onSave: () => {},
  onClose: () => {},
};

function project(name: string, ref: string): DayliteProjectSummary {
  return { self: ref, name, status: "in_progress" };
}

describe("AssignmentModal", () => {
  it("renders nothing when closed", () => {
    const html = renderToStaticMarkup(
      <AssignmentModal {...baseProps} isOpen={false} assignment={null} />,
    );
    expect(html).toBe("");
  });

  it("create mode: renders the project filter input and save button", () => {
    const html = renderToStaticMarkup(
      <AssignmentModal {...baseProps} isOpen assignment={null} />,
    );

    expect(html).toContain("<dialog");
    expect(html).toContain("Einsatz erstellen");
    expect(html).toContain('role="combobox"');
    expect(html).toContain("Projekt suchen...");
    expect(html).not.toContain("<select");
    expect(html).toContain("Speichern");
    expect(html).not.toContain("Löschen");
  });

  it("create mode: starts with an empty result list (empty default state)", () => {
    const html = renderToStaticMarkup(
      <AssignmentModal {...baseProps} isOpen assignment={null} />,
    );

    expect(html).not.toContain('id="assignment-project-results"');
  });

  it("edit mode: shows the selected project and the delete button", () => {
    const existingAssignment: CalendarCellEvent = {
      uid: "uid-1",
      kind: "assignment",
      title: "Projekt Alpha",
      projectStatus: "in_progress",
      projectRef: "/v1/projects/1",
      date: "2026-05-06",
      startTime: "08:00",
      endTime: "16:00",
      href: "/calendars/user/cal/uid-1.ics",
    };

    const html = renderToStaticMarkup(
      <AssignmentModal {...baseProps} isOpen assignment={existingAssignment} />,
    );

    expect(html).toContain("<dialog");
    expect(html).toContain("Einsatz bearbeiten");
    expect(html).toContain('role="combobox"');
    expect(html).toContain("Ausgewählt:");
    expect(html).toContain("Projekt Alpha");
    expect(html).toContain("Speichern");
    expect(html).toContain("Löschen");
  });

  it("unsaved changes dialog renders when closing modal with dirty state", () => {
    const html = renderToStaticMarkup(
      <AssignmentModal
        {...baseProps}
        isOpen
        assignment={null}
        showUnsavedConfirm
      />,
    );

    expect(html).toContain("Ungespeicherte Änderungen");
    expect(html).toContain("Verwerfen");
    expect(html).toContain("Weiterbearbeiten");
  });

  it("delete confirmation dialog renders correctly", () => {
    const existingAssignment: CalendarCellEvent = {
      uid: "uid-2",
      kind: "assignment",
      title: "Projekt Beta",
      projectStatus: "new_status",
      projectRef: "/v1/projects/2",
      date: "2026-05-06",
      startTime: null,
      endTime: null,
      href: "/calendars/user/cal/uid-2.ics",
    };

    const html = renderToStaticMarkup(
      <AssignmentModal
        {...baseProps}
        isOpen
        assignment={existingAssignment}
        showDeleteConfirm
      />,
    );

    expect(html).toContain("Einsatz löschen");
    expect(html).toContain("Endgültig löschen");
    expect(html).toContain("Abbrechen");
  });
});

// ── 6.1 – result list display vs empty default state ──────────────────────────
describe("ProjectResultList", () => {
  it("renders the filtered projects as selectable options", () => {
    const html = renderToStaticMarkup(
      <ProjectResultList
        projects={[
          project("Projekt Nord", "/v1/projects/10"),
          project("Projekt Süd", "/v1/projects/11"),
        ]}
        highlightedIndex={-1}
        onSelect={() => {}}
      />,
    );

    expect(html).toContain('id="assignment-project-results"');
    expect(html).toContain("Projekt Nord");
    expect(html).toContain("Projekt Süd");
    expect(html.match(/<button/g)).toHaveLength(2);
  });

  it("renders nothing for an empty result list (empty default state)", () => {
    const html = renderToStaticMarkup(
      <ProjectResultList
        projects={[]}
        highlightedIndex={-1}
        onSelect={() => {}}
      />,
    );

    expect(html).toBe("");
  });

  // ── 6.3 – keyboard selection highlight ──────────────────────────────────────
  it("marks the highlighted option as selected", () => {
    const html = renderToStaticMarkup(
      <ProjectResultList
        projects={[
          project("Projekt Nord", "/v1/projects/10"),
          project("Projekt Süd", "/v1/projects/11"),
        ]}
        highlightedIndex={1}
        onSelect={() => {}}
      />,
    );

    expect(html).toContain('aria-current="true"');
    expect(html).toContain('aria-current="false"');
  });
});

// ── 6.3 – arrow key navigation over the displayed list ────────────────────────
describe("nextHighlightIndex", () => {
  it("moves down from the unhighlighted state to the first item", () => {
    expect(nextHighlightIndex(-1, 3, 1)).toBe(0);
  });

  it("moves down and up within bounds", () => {
    expect(nextHighlightIndex(0, 3, 1)).toBe(1);
    expect(nextHighlightIndex(1, 3, -1)).toBe(0);
  });

  it("clamps at the list boundaries", () => {
    expect(nextHighlightIndex(2, 3, 1)).toBe(2);
    expect(nextHighlightIndex(0, 3, -1)).toBe(0);
  });

  it("stays unhighlighted for an empty list", () => {
    expect(nextHighlightIndex(-1, 0, 1)).toBe(-1);
  });
});

// ── 6.5 – Escape precedence (clear vs close) ──────────────────────────────────
describe("resolveEscapeAction", () => {
  it("clears a non-empty filter instead of closing", () => {
    expect(resolveEscapeAction("Nord")).toBe("clear");
  });

  it("falls through to the modal close flow when the filter is empty", () => {
    expect(resolveEscapeAction("")).toBe("close");
  });
});
