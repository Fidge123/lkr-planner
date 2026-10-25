import { beforeEach, describe, expect, it, mock } from "bun:test";
import { fireEvent, screen, waitFor } from "@testing-library/react";
import type {
  CalendarCellEvent,
  DayliteProjectSummary,
  UpdateAssignmentInput,
} from "../../generated/tauri";
import { resetLastAssignedProject } from "../../services/assignment-suggestions";
import { commandFailed, commandOk } from "../../test/app-error";
import { mockCommands } from "../../test/mock-commands";
import { renderComponent } from "../../test/render-component";
import { AssignmentModal } from "./assignment-modal";
import {
  nextHighlightIndex,
  resolveWriteIntent,
} from "./assignment-modal-logic";
import { ProjectResultList, SuggestionEmptyState } from "./project-result-list";

const mockUpdateAssignment = mock((_input: UpdateAssignmentInput) =>
  commandOk(null),
);
const mockDeleteAssignment = mock((_href: string, _override: boolean) =>
  commandOk(null),
);
const onSave = mock(() => {});
const onClose = mock(() => {});

function project(name: string, ref: string): DayliteProjectSummary {
  return { self: ref, name, status: "in_progress" };
}

const overdueProject = project("Projekt Nord", "/v1/projects/10");

mockCommands({
  dayliteQueryOverdueProjects: () => commandOk([overdueProject]),
  dayliteProjectCategoryColors: () => commandOk({}),
  updateAssignment: mockUpdateAssignment,
  deleteAssignment: mockDeleteAssignment,
});

const plannableAssignment: CalendarCellEvent = {
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

const fixedAssignment: CalendarCellEvent = {
  ...plannableAssignment,
  uid: "uid-9",
  title: "Projekt Fix",
  projectCategory: "Termin FIX geplant",
  projectRef: "/v1/projects/9",
  href: "/calendars/user/cal/uid-9.ics",
};

function renderModal(assignment: CalendarCellEvent | null) {
  return renderComponent(
    <AssignmentModal
      assignment={assignment}
      employeeReference="ref-123"
      date="2026-05-06"
      onSave={onSave}
      onClose={onClose}
    />,
  );
}

const button = (name: string) =>
  screen.getByRole<HTMLButtonElement>("button", { name });
const projectInput = () => screen.getByRole<HTMLInputElement>("combobox");

describe("AssignmentModal", () => {
  beforeEach(() => {
    resetLastAssignedProject();
    for (const fn of [
      mockUpdateAssignment,
      mockDeleteAssignment,
      onSave,
      onClose,
    ]) {
      fn.mockClear();
    }
  });

  it("create mode: renders the project filter input and save button", () => {
    const html = renderModal(null).container.innerHTML;

    expect(screen.getByText("Einsatz erstellen")).toBeDefined();
    expect(projectInput().placeholder).toBe("Projekt suchen...");
    expect(html).not.toContain("<select");
    expect(button("Speichern").disabled).toBe(true);
    expect(screen.queryByRole("button", { name: "Löschen" })).toBeNull();
  });

  it("create mode: reserves the suggestion rows until the suggestions are loaded", async () => {
    const { container } = renderModal(null);

    expect(container.innerHTML.match(/skeleton/g)).toHaveLength(5);
    expect(await screen.findByText("Projekt Nord")).toBeDefined();
    expect(container.innerHTML).not.toContain("skeleton");
  });

  it("edit mode: shows the selected project and the delete button", () => {
    renderModal(plannableAssignment);

    expect(screen.getByText("Einsatz bearbeiten")).toBeDefined();
    expect(screen.getByText("Projekt Alpha").textContent).toBe("Projekt Alpha");
    expect(button("Speichern").disabled).toBe(false);
    expect(button("Löschen").disabled).toBe(false);
    expect(screen.queryByText("Bearbeitung entsperren")).toBeNull();
  });

  it("asks before discarding a selected project and keeps editing on request", async () => {
    renderModal(null);
    fireEvent.click(await screen.findByText("Projekt Nord"));

    fireEvent.click(button("Abbrechen"));

    expect(screen.getByText("Ungespeicherte Änderungen")).toBeDefined();
    fireEvent.click(button("Weiterbearbeiten"));
    expect(screen.getByText("Einsatz erstellen")).toBeDefined();
    fireEvent.click(button("Abbrechen"));
    fireEvent.click(button("Verwerfen"));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("closes on Escape, but asks first once a project was selected", async () => {
    const pressEscape = () =>
      fireEvent(
        screen.getByRole("dialog"),
        new Event("cancel", { cancelable: true }),
      );
    renderModal(null);

    pressEscape();
    expect(onClose).toHaveBeenCalledTimes(1);

    fireEvent.click(await screen.findByText("Projekt Nord"));
    pressEscape();
    expect(screen.getByText("Ungespeicherte Änderungen")).toBeDefined();
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("deletes only after confirmation", async () => {
    renderModal(plannableAssignment);

    fireEvent.click(button("Löschen"));
    expect(screen.getByText("Einsatz löschen")).toBeDefined();
    expect(mockDeleteAssignment).not.toHaveBeenCalled();
    fireEvent.click(button("Endgültig löschen"));

    await waitFor(() =>
      expect(onSave).toHaveBeenCalledWith({ kind: "delete" }),
    );
    expect(mockDeleteAssignment).toHaveBeenCalledWith(
      plannableAssignment.href,
      false,
    );
  });

  it("shows the backend's rejection in the delete confirmation", async () => {
    const rejection =
      "Dieser Termin ist als 'Termin FIX geplant' gesperrt und kann nicht geändert oder gelöscht werden.";
    mockDeleteAssignment.mockImplementationOnce(() => commandFailed(rejection));
    renderModal(plannableAssignment);

    fireEvent.click(button("Löschen"));
    fireEvent.click(button("Endgültig löschen"));

    expect(await screen.findByText(rejection)).toBeDefined();
    expect(onSave).not.toHaveBeenCalled();
  });

  it("locks the picker, save and delete for a fixed appointment", () => {
    renderModal(fixedAssignment);

    expect(screen.getByText(/Termin FIX geplant/)).toBeDefined();
    expect(
      screen.getByRole<HTMLInputElement>("checkbox", {
        name: "Bearbeitung entsperren",
      }).checked,
    ).toBe(false);
    expect(projectInput().disabled).toBe(true);
    expect(button("Löschen").disabled).toBe(true);
    expect(button("Speichern").disabled).toBe(true);
  });

  it("saves an unlocked fixed appointment with the override", async () => {
    renderModal(fixedAssignment);

    fireEvent.click(screen.getByText("Bearbeitung entsperren"));
    expect(projectInput().disabled).toBe(false);
    fireEvent.click(button("Speichern"));

    await waitFor(() => expect(onSave).toHaveBeenCalled());
    expect(mockUpdateAssignment).toHaveBeenCalledWith(
      expect.objectContaining({
        href: fixedAssignment.href,
        overrideProtection: true,
      }),
    );
  });

  it("deletes an unlocked fixed appointment with the override", async () => {
    renderModal(fixedAssignment);

    fireEvent.click(screen.getByText("Bearbeitung entsperren"));
    fireEvent.click(button("Löschen"));
    fireEvent.click(button("Endgültig löschen"));

    await waitFor(() => expect(onSave).toHaveBeenCalled());
    expect(mockDeleteAssignment).toHaveBeenCalledWith(
      fixedAssignment.href,
      true,
    );
  });

  it("locks a fixed appointment again when the modal reopens", () => {
    const { unmount } = renderModal(fixedAssignment);
    fireEvent.click(screen.getByText("Bearbeitung entsperren"));
    unmount();

    renderModal(fixedAssignment);

    expect(projectInput().disabled).toBe(true);
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

describe("resolveWriteIntent", () => {
  it("refuses an edit without a resource URL instead of falling back to create", () => {
    expect(resolveWriteIntent(true, null)).toBe("missing-href");
    expect(resolveWriteIntent(true, undefined)).toBe("missing-href");
    expect(resolveWriteIntent(true, "")).toBe("missing-href");
  });
});
