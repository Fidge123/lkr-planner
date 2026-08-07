import { describe, expect, it, mock } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import type {
  CalendarCellEvent,
  DayliteProjectSummary,
} from "../../generated/tauri";
import { AssignmentModal } from "./assignment-modal";
import {
  nextHighlightIndex,
  resolveDisplayedProjects,
  resolveEscapeAction,
  resolveSaveAction,
  resolveWriteIntent,
} from "./assignment-modal-logic";
import { ProjectResultList, SuggestionEmptyState } from "./project-result-list";

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
      categoryColor: null,
      projectRef: "/v1/projects/1",
      date: "2026-05-06",
      startTime: "08:00",
      endTime: "16:00",
      href: "/calendars/user/cal/uid-1.ics",
      orderIndex: null,
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
      categoryColor: null,
      projectRef: "/v1/projects/2",
      date: "2026-05-06",
      startTime: null,
      endTime: null,
      href: "/calendars/user/cal/uid-2.ics",
      orderIndex: null,
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
    expect(html).toContain("bg-primary");
  });

  it("marks each result with a dot in its daylite category color", () => {
    const html = renderToStaticMarkup(
      <ProjectResultList
        projects={[
          { ...project("Projekt Nord", "/v1/projects/10"), category: "Bau" },
          {
            ...project("Projekt Süd", "/v1/projects/11"),
            category: "Ohne Farbe",
          },
        ]}
        categoryColors={{ Bau: "#8bc34a" }}
        highlightedIndex={-1}
        onSelect={() => {}}
      />,
    );

    expect(html).toContain("background-color:#8bc34a");
    expect(html.match(/rounded-full/g)).toHaveLength(2);
    expect(html).toContain("bg-base-content/30");
  });

  it("keeps the category dot colored on the selected row", () => {
    const html = renderToStaticMarkup(
      <ProjectResultList
        projects={[
          { ...project("Projekt Nord", "/v1/projects/10"), category: "Bau" },
        ]}
        categoryColors={{ Bau: "#8bc34a" }}
        highlightedIndex={0}
        onSelect={() => {}}
      />,
    );

    expect(html).toContain("bg-primary text-primary-content");
    expect(html).toContain("background-color:#8bc34a");
  });

  it("renders a project name with hard line breaks as a single line", () => {
    const html = renderToStaticMarkup(
      <ProjectResultList
        projects={[project("DARAMIC LLC,\n Norderstedt", "/v1/projects/10")]}
        highlightedIndex={-1}
        onSelect={() => {}}
      />,
    );

    expect(html).toContain("DARAMIC LLC, Norderstedt");
    expect(html).toContain("truncate");
  });
});

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
    expect(resolveDisplayedProjects("", suggestions, results)).toBe(
      suggestions,
    );
  });
});

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

describe("resolveEscapeAction", () => {
  it("clears a non-empty filter instead of closing", () => {
    expect(resolveEscapeAction("Nord")).toBe("clear");
  });

  it("falls through to the modal close flow when the filter is empty", () => {
    expect(resolveEscapeAction("")).toBe("close");
  });
});

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

describe("resolveWriteIntent", () => {
  it("creates when not editing", () => {
    expect(resolveWriteIntent(false, null)).toBe("create");
    expect(resolveWriteIntent(false, "/cal/uid-1.ics")).toBe("create");
  });

  it("updates an edited assignment that has a resource URL", () => {
    expect(resolveWriteIntent(true, "/cal/uid-1.ics")).toBe("update");
  });

  it("refuses an edit without a resource URL instead of falling back to create", () => {
    expect(resolveWriteIntent(true, null)).toBe("missing-href");
    expect(resolveWriteIntent(true, undefined)).toBe("missing-href");
    expect(resolveWriteIntent(true, "")).toBe("missing-href");
  });
});
