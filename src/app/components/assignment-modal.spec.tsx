import { describe, expect, it, mock } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import type { CalendarCellEvent } from "../../generated/tauri";
import { AssignmentModal } from "./assignment-modal";

mock.module("../../services/assignment-project-picker", () => ({
  loadProjectsForAssignmentPicker: mock(() => Promise.resolve([])),
}));

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

describe("AssignmentModal", () => {
  it("renders nothing when closed", () => {
    const html = renderToStaticMarkup(
      <AssignmentModal {...baseProps} isOpen={false} assignment={null} />,
    );
    expect(html).toBe("");
  });

  it("create mode: renders empty project picker and save button", () => {
    const html = renderToStaticMarkup(
      <AssignmentModal {...baseProps} isOpen assignment={null} />,
    );

    expect(html).toContain("<dialog");
    expect(html).toContain("Einsatz erstellen");
    expect(html).toContain("<select");
    expect(html).toContain("Speichern");
    expect(html).not.toContain("Löschen");
  });

  it("edit mode: renders project picker and delete button", () => {
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
    expect(html).toContain("<select");
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
