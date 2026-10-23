import { describe, expect, it } from "bun:test";
import type {
  CalendarCellEvent,
  DayliteProjectSummary,
} from "../../generated/tauri";
import { renderComponent } from "../../test/render-component";
import { AssignmentModal } from "./assignment-modal";
import {
  isProtectedAssignment,
  nextHighlightIndex,
  resolveDisplayedProjects,
  resolveEscapeAction,
  resolveSaveAction,
  resolveWriteIntent,
} from "./assignment-modal-logic";
import { DeleteConfirmDialog } from "./delete-confirm-dialog";
import {
  ProjectResultList,
  ProjectResultPlaceholder,
  SuggestionEmptyState,
} from "./project-result-list";

const baseProps = {
  employeeReference: "ref-123",
  date: "2026-05-06",
  onSave: () => {},
  onClose: () => {},
};

const fixedAssignment: CalendarCellEvent = {
  uid: "uid-9",
  kind: "assignment",
  title: "Projekt Fix",
  projectStatus: "in_progress",
  projectCategory: "Termin FIX geplant",
  projectRef: "/v1/projects/9",
  date: "2026-05-06",
  startTime: "08:00",
  endTime: "16:00",
  href: "/calendars/user/cal/uid-9.ics",
  orderIndex: null,
  projectPending: false,
};

function project(name: string, ref: string): DayliteProjectSummary {
  return { self: ref, name, status: "in_progress" };
}

describe("AssignmentModal", () => {
  it("create mode: renders the project filter input and save button", () => {
    const html = renderComponent(
      <AssignmentModal {...baseProps} assignment={null} />,
    ).container.innerHTML;

    expect(html).toContain("<dialog");
    expect(html).toContain("Einsatz erstellen");
    expect(html).toContain('role="combobox"');
    expect(html).toContain("Projekt suchen...");
    expect(html).not.toContain("<select");
    expect(html).toContain("Speichern");
    expect(html).not.toContain("Löschen");
  });

  it("create mode: starts with an empty result list (empty default state)", () => {
    const html = renderComponent(
      <AssignmentModal {...baseProps} assignment={null} />,
    ).container.innerHTML;

    expect(html).not.toContain('id="assignment-project-results"');
  });

  it("reserves the suggestion rows until the suggestions are loaded", () => {
    const html = renderComponent(
      <AssignmentModal {...baseProps} assignment={null} />,
    ).container.innerHTML;

    expect(html.match(/skeleton/g)).toHaveLength(5);
  });

  it("edit mode: shows the selected project and the delete button", () => {
    const existingAssignment: CalendarCellEvent = {
      uid: "uid-1",
      kind: "assignment",
      title: "Projekt Alpha",
      projectStatus: "in_progress",
      projectCategory: null,
      projectRef: "/v1/projects/1",
      date: "2026-05-06",
      startTime: "08:00",
      endTime: "16:00",
      href: "/calendars/user/cal/uid-1.ics",
      orderIndex: null,
      projectPending: false,
    };

    const html = renderComponent(
      <AssignmentModal {...baseProps} assignment={existingAssignment} />,
    ).container.innerHTML;

    expect(html).toContain("<dialog");
    expect(html).toContain("Einsatz bearbeiten");
    expect(html).toContain('role="combobox"');
    expect(html).toContain("Ausgewählt:");
    expect(html).toContain("Projekt Alpha");
    expect(html).toContain("Speichern");
    expect(html).toContain("Löschen");
  });

  it("edit mode: locks the picker, save and delete for a fixed appointment", () => {
    const html = renderComponent(
      <AssignmentModal {...baseProps} assignment={fixedAssignment} />,
    ).container.innerHTML;

    expect(html).toContain("Termin FIX geplant");
    expect(html).toContain("Bearbeitung entsperren");
    expect(html.match(/<input[^>]*type="checkbox"[^>]*checked/)).toBeNull();
    expect(
      html.match(/<input(?=[^>]*role="combobox")[^>]*disabled/),
    ).not.toBeNull();
    expect(html.match(/<button[^>]*disabled[^>]*>Löschen/)).not.toBeNull();
    expect(html.match(/<button[^>]*disabled[^>]*>Speichern/)).not.toBeNull();
  });

  it("edit mode: keeps save and delete enabled for a plannable assignment", () => {
    const assignment: CalendarCellEvent = {
      uid: "uid-1",
      kind: "assignment",
      title: "Projekt Alpha",
      projectStatus: "in_progress",
      projectCategory: null,
      projectRef: "/v1/projects/1",
      date: "2026-05-06",
      startTime: "08:00",
      endTime: "16:00",
      href: "/calendars/user/cal/uid-1.ics",
      orderIndex: null,
      projectPending: false,
    };

    const html = renderComponent(
      <AssignmentModal {...baseProps} assignment={assignment} />,
    ).container.innerHTML;

    expect(html).not.toContain("Termin FIX geplant");
    expect(html).not.toContain("Bearbeitung entsperren");
    expect(html.match(/<button[^>]*disabled[^>]*>Löschen/)).toBeNull();
    expect(html.match(/<button[^>]*disabled[^>]*>Speichern/)).toBeNull();
  });

  it("unsaved changes dialog renders when closing modal with dirty state", () => {
    const html = renderComponent(
      <AssignmentModal {...baseProps} assignment={null} showUnsavedConfirm />,
    ).container.innerHTML;

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
      projectCategory: null,
      projectRef: "/v1/projects/2",
      date: "2026-05-06",
      startTime: null,
      endTime: null,
      href: "/calendars/user/cal/uid-2.ics",
      orderIndex: null,
      projectPending: false,
    };

    const html = renderComponent(
      <AssignmentModal
        {...baseProps}
        assignment={existingAssignment}
        showDeleteConfirm
      />,
    ).container.innerHTML;

    expect(html).toContain("Einsatz löschen");
    expect(html).toContain("Endgültig löschen");
    expect(html).toContain("Abbrechen");
  });
});

describe("ProjectResultList", () => {
  it("renders the filtered projects as selectable options", () => {
    const html = renderComponent(
      <ProjectResultList
        projects={[
          project("Projekt Nord", "/v1/projects/10"),
          project("Projekt Süd", "/v1/projects/11"),
        ]}
        highlightedIndex={-1}
        onSelect={() => {}}
      />,
    ).container.innerHTML;

    expect(html).toContain('id="assignment-project-results"');
    expect(html).toContain("Projekt Nord");
    expect(html).toContain("Projekt Süd");
    expect(html.match(/<button/g)).toHaveLength(2);
  });

  it("renders nothing for an empty result list (empty default state)", () => {
    const html = renderComponent(
      <ProjectResultList
        projects={[]}
        highlightedIndex={-1}
        onSelect={() => {}}
      />,
    ).container.innerHTML;

    expect(html).toBe("");
  });

  it("marks the highlighted option as selected", () => {
    const html = renderComponent(
      <ProjectResultList
        projects={[
          project("Projekt Nord", "/v1/projects/10"),
          project("Projekt Süd", "/v1/projects/11"),
        ]}
        highlightedIndex={1}
        onSelect={() => {}}
      />,
    ).container.innerHTML;

    expect(html).toContain('aria-current="true"');
    expect(html).toContain('aria-current="false"');
    expect(html).toContain("bg-primary");
  });

  it("marks each result with a dot in its daylite category color", () => {
    const html = renderComponent(
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
    ).container.innerHTML;

    expect(html).toContain("background-color: #8bc34a");
    expect(html.match(/rounded-full/g)).toHaveLength(2);
    expect(html).toContain("bg-base-content/30");
  });

  it("keeps the category dot colored on the selected row", () => {
    const html = renderComponent(
      <ProjectResultList
        projects={[
          { ...project("Projekt Nord", "/v1/projects/10"), category: "Bau" },
        ]}
        categoryColors={{ Bau: "#8bc34a" }}
        highlightedIndex={0}
        onSelect={() => {}}
      />,
    ).container.innerHTML;

    expect(html).toContain("bg-primary text-primary-content");
    expect(html).toContain("background-color: #8bc34a");
  });

  it("reserves rows of the same height as the loaded results", () => {
    const placeholder = renderComponent(<ProjectResultPlaceholder />).container
      .innerHTML;
    const loaded = renderComponent(
      <ProjectResultList
        projects={[project("Projekt Nord", "/v1/projects/10")]}
        highlightedIndex={-1}
        onSelect={() => {}}
      />,
    ).container.innerHTML;

    expect(placeholder.match(/h-8/g)).toHaveLength(5);
    expect(loaded).toContain("h-8");
  });

  it("renders a project name with hard line breaks as a single line", () => {
    const html = renderComponent(
      <ProjectResultList
        projects={[project("DARAMIC LLC,\n Norderstedt", "/v1/projects/10")]}
        highlightedIndex={-1}
        onSelect={() => {}}
      />,
    ).container.innerHTML;

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
});

describe("SuggestionEmptyState", () => {
  it("shows the German message when no suggestions are available", () => {
    const html = renderComponent(
      <SuggestionEmptyState filter="" suggestionsLoaded suggestionCount={0} />,
    ).container.innerHTML;

    expect(html).toContain("Keine Vorschläge verfügbar");
  });

  it("shows nothing while the suggestions are still loading", () => {
    const html = renderComponent(
      <SuggestionEmptyState
        filter=""
        suggestionsLoaded={false}
        suggestionCount={0}
      />,
    ).container.innerHTML;

    expect(html).toBe("");
  });

  it("shows nothing when suggestions are available", () => {
    const html = renderComponent(
      <SuggestionEmptyState filter="" suggestionsLoaded suggestionCount={3} />,
    ).container.innerHTML;

    expect(html).toBe("");
  });

  it("shows nothing while a filter is set", () => {
    const html = renderComponent(
      <SuggestionEmptyState
        filter="Nord"
        suggestionsLoaded
        suggestionCount={0}
      />,
    ).container.innerHTML;

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

describe("isProtectedAssignment", () => {
  it("protects an assignment whose project is a fixed appointment", () => {
    expect(isProtectedAssignment("Termin FIX geplant")).toBe(true);
  });

  it("leaves an assignment of any other project category plannable", () => {
    expect(isProtectedAssignment("Liefertermin bekannt")).toBe(false);
  });

  it("leaves an assignment without a resolved category plannable", () => {
    expect(isProtectedAssignment(null)).toBe(false);
  });
});

describe("DeleteConfirmDialog", () => {
  it("shows the backend's rejection message for a stale delete attempt", () => {
    const rejection =
      "Dieser Termin ist als 'Termin FIX geplant' gesperrt und kann nicht geändert oder gelöscht werden.";

    const html = renderComponent(
      <DeleteConfirmDialog
        isDeleting={false}
        errorMessage={rejection}
        onCancel={() => {}}
        onConfirm={() => {}}
        onRequestClose={() => {}}
      />,
    ).container.innerHTML;

    expect(html).toContain("Termin FIX geplant");
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
