import { describe, expect, it, mock } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import type {
  CalendarCellEvent,
  DayliteProjectSummary,
} from "../../generated/tauri";
import { combineSuggestions } from "../../services/assignment-suggestions";
import {
  AssignmentModal,
  nextHighlightIndex,
  ProjectResultList,
  resolveDisplayedProjects,
  resolveEscapeAction,
  resolveSaveAction,
  SuggestionEmptyState,
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
    // The highlight uses explicit utility classes so it stays visible
    // independent of DaisyUI's menu-active styling.
    expect(html).toContain("bg-primary");
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

// ── BL-031 5.1 / 5.2 / 5.3 – default suggestions in the result list ───────────
describe("default suggestions rendering", () => {
  const overdue = [
    project("Projekt 10", "/v1/projects/10"),
    project("Projekt 11", "/v1/projects/11"),
    project("Projekt 12", "/v1/projects/12"),
    project("Projekt 13", "/v1/projects/13"),
    project("Projekt 14", "/v1/projects/14"),
  ];

  it("renders the recent project first, followed by overdue projects", () => {
    const recent = project("Projekt Zuletzt", "/v1/projects/99");

    const html = renderToStaticMarkup(
      <ProjectResultList
        projects={combineSuggestions(recent, overdue)}
        highlightedIndex={-1}
        onSelect={() => {}}
      />,
    );

    expect(html.indexOf("Projekt Zuletzt")).toBeGreaterThan(-1);
    expect(html.indexOf("Projekt Zuletzt")).toBeLessThan(
      html.indexOf("Projekt 10"),
    );
  });

  it("renders at most 5 suggestions", () => {
    const recent = project("Projekt Zuletzt", "/v1/projects/99");

    const html = renderToStaticMarkup(
      <ProjectResultList
        projects={combineSuggestions(recent, overdue)}
        highlightedIndex={-1}
        onSelect={() => {}}
      />,
    );

    expect(html.match(/<button/g)).toHaveLength(5);
    expect(html).not.toContain("Projekt 14");
  });

  it("renders a recent project that is also overdue only once", () => {
    const recent = project("Projekt 11", "/v1/projects/11");

    const html = renderToStaticMarkup(
      <ProjectResultList
        projects={combineSuggestions(recent, overdue)}
        highlightedIndex={-1}
        onSelect={() => {}}
      />,
    );

    expect(html.match(/Projekt 11/g)).toHaveLength(1);
    expect(html.match(/<button/g)).toHaveLength(5);
  });
});

// ── BL-031 5.7 – clearing the filter restores the default suggestions ─────────
describe("resolveDisplayedProjects", () => {
  const suggestions = [project("Projekt Zuletzt", "/v1/projects/99")];
  const results = [project("Projekt Nord", "/v1/projects/10")];

  it("shows the default suggestions for an empty filter", () => {
    expect(resolveDisplayedProjects("", suggestions, results)).toBe(
      suggestions,
    );
  });

  it("shows the live search results while a filter is set", () => {
    expect(resolveDisplayedProjects("Nord", suggestions, results)).toBe(
      results,
    );
  });

  it("restores the suggestions after the filter is cleared", () => {
    expect(resolveDisplayedProjects("Nord", suggestions, results)).toBe(
      results,
    );
    // Escape or manual clearing empties the filter (see resolveEscapeAction).
    expect(resolveDisplayedProjects("", suggestions, results)).toBe(
      suggestions,
    );
  });
});

// ── BL-031 5.6 – empty state message display ──────────────────────────────────
describe("SuggestionEmptyState", () => {
  it("shows the German message when no suggestions are available", () => {
    const html = renderToStaticMarkup(
      <SuggestionEmptyState filter="" suggestionsLoaded suggestionCount={0} />,
    );

    expect(html).toContain("Keine Vorschläge verfügbar");
  });

  it("shows nothing while the suggestions are still loading", () => {
    const html = renderToStaticMarkup(
      <SuggestionEmptyState
        filter=""
        suggestionsLoaded={false}
        suggestionCount={0}
      />,
    );

    expect(html).toBe("");
  });

  it("shows nothing when suggestions are available", () => {
    const html = renderToStaticMarkup(
      <SuggestionEmptyState filter="" suggestionsLoaded suggestionCount={3} />,
    );

    expect(html).toBe("");
  });

  it("shows nothing while a filter is set", () => {
    const html = renderToStaticMarkup(
      <SuggestionEmptyState
        filter="Nord"
        suggestionsLoaded
        suggestionCount={0}
      />,
    );

    expect(html).toBe("");
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

// ── bl-033 1.1 / 1.3 – only a create carries a next-day ghost payload ─────────
describe("resolveSaveAction", () => {
  it("builds a create action carrying the saved project", () => {
    expect(
      resolveSaveAction(false, "2026-05-06", "/v1/projects/1", "Projekt Nord"),
    ).toEqual({
      kind: "create",
      date: "2026-05-06",
      projectRef: "/v1/projects/1",
      projectName: "Projekt Nord",
    });
  });

  it("builds a bare edit action regardless of the selected project", () => {
    expect(
      resolveSaveAction(true, "2026-05-06", "/v1/projects/1", "Projekt Nord"),
    ).toEqual({ kind: "edit" });
  });
});
