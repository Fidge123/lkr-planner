import { describe, expect, it, mock } from "bun:test";
import type {
  AppointmentDragPayload,
  DropCellTarget,
} from "./use-appointment-drag";
import {
  computeEdgeZone,
  decideDropAction,
  EdgeHoverNavigator,
  performDrop,
} from "./use-appointment-drag";

const payload: AppointmentDragPayload = {
  uid: "uid-1",
  href: "/calendars/emp-a/uid-1.ics",
  projectRef: "/v1/projects/42",
  projectName: "Projekt Nord",
  employeeRef: "/v1/contacts/1",
  date: "2026-07-06",
  title: "Projekt Nord",
  color: "bg-primary",
  startTime: "08:00",
  endTime: "16:00",
};

type CommandResult<T> =
  | { status: "ok"; data: T }
  | { status: "error"; error: string };
type MoveData =
  | { kind: "moved"; newHref: string }
  | { kind: "sourceDeleteFailed"; newHref: string; sourceHref: string };

const okDeps = () => ({
  hasCalendar: mock((_employeeRef: string) => true),
  updateAssignment: mock(
    async (): Promise<CommandResult<null>> => ({ status: "ok", data: null }),
  ),
  moveAssignment: mock(
    async (): Promise<CommandResult<MoveData>> => ({
      status: "ok",
      data: { kind: "moved", newHref: "/calendars/emp-b/new.ics" },
    }),
  ),
});

describe("decideDropAction", () => {
  it("is a no-op for the originating cell", () => {
    const target: DropCellTarget = {
      employeeRef: payload.employeeRef,
      date: payload.date,
    };
    expect(decideDropAction(payload, target)).toEqual({ kind: "none" });
  });

  it("reschedules on the same employee for a different date", () => {
    const target: DropCellTarget = {
      employeeRef: payload.employeeRef,
      date: "2026-07-08",
    };
    expect(decideDropAction(payload, target)).toEqual({ kind: "reschedule" });
  });

  it("moves when the target belongs to a different employee", () => {
    const target: DropCellTarget = {
      employeeRef: "/v1/contacts/2",
      date: payload.date,
    };
    expect(decideDropAction(payload, target)).toEqual({ kind: "move" });
  });
});

describe("performDrop", () => {
  it("makes no persistence call when dropped on the originating cell", async () => {
    const deps = okDeps();
    const outcome = await performDrop(
      payload,
      { employeeRef: payload.employeeRef, date: payload.date },
      deps,
    );

    expect(outcome).toEqual({ kind: "none" });
    expect(deps.updateAssignment).not.toHaveBeenCalled();
    expect(deps.moveAssignment).not.toHaveBeenCalled();
  });

  it("reschedules via updateAssignment within the same employee", async () => {
    const deps = okDeps();
    const outcome = await performDrop(
      payload,
      { employeeRef: payload.employeeRef, date: "2026-07-08" },
      deps,
    );

    expect(outcome).toEqual({ kind: "done" });
    expect(deps.updateAssignment).toHaveBeenCalledWith(
      payload.href,
      payload.uid,
      "2026-07-08",
      payload.projectRef,
      payload.projectName,
    );
    expect(deps.moveAssignment).not.toHaveBeenCalled();
  });

  it("moves via moveAssignment to a different employee", async () => {
    const deps = okDeps();
    const outcome = await performDrop(
      payload,
      { employeeRef: "/v1/contacts/2", date: "2026-07-08" },
      deps,
    );

    expect(outcome).toEqual({ kind: "done" });
    expect(deps.moveAssignment).toHaveBeenCalledWith(
      payload.href,
      "/v1/contacts/2",
      "2026-07-08",
      payload.projectRef,
      payload.projectName,
    );
    expect(deps.updateAssignment).not.toHaveBeenCalled();
  });

  it("rejects a move to an employee without a configured calendar", async () => {
    const deps = { ...okDeps(), hasCalendar: mock((_ref: string) => false) };
    const outcome = await performDrop(
      payload,
      { employeeRef: "/v1/contacts/2", date: "2026-07-08" },
      deps,
    );

    expect(outcome).toEqual({
      kind: "error",
      message: "Kein Kalender für diesen Mitarbeiter konfiguriert.",
    });
    expect(deps.moveAssignment).not.toHaveBeenCalled();
  });

  it("surfaces a partial move so the caller can reconcile", async () => {
    const deps = {
      ...okDeps(),
      moveAssignment: mock(
        async (): Promise<CommandResult<MoveData>> => ({
          status: "ok",
          data: {
            kind: "sourceDeleteFailed",
            newHref: "/calendars/emp-b/new.ics",
            sourceHref: payload.href,
          },
        }),
      ),
    };
    const outcome = await performDrop(
      payload,
      { employeeRef: "/v1/contacts/2", date: "2026-07-08" },
      deps,
    );

    expect(outcome).toEqual({
      kind: "partialMove",
      newHref: "/calendars/emp-b/new.ics",
      sourceHref: payload.href,
    });
  });

  it("returns the backend error message when the command fails", async () => {
    const deps = {
      ...okDeps(),
      updateAssignment: mock(
        async (): Promise<CommandResult<null>> => ({
          status: "error",
          error: "Kalenderserver antwortete mit HTTP 500",
        }),
      ),
    };
    const outcome = await performDrop(
      payload,
      { employeeRef: payload.employeeRef, date: "2026-07-08" },
      deps,
    );

    expect(outcome).toEqual({
      kind: "error",
      message: "Kalenderserver antwortete mit HTTP 500",
    });
  });
});

describe("computeEdgeZone", () => {
  it("detects the left and right edge bands", () => {
    expect(computeEdgeZone(10, 1000, 48)).toBe("left");
    expect(computeEdgeZone(990, 1000, 48)).toBe("right");
  });

  it("returns null in the middle of the grid", () => {
    expect(computeEdgeZone(500, 1000, 48)).toBeNull();
  });
});

describe("EdgeHoverNavigator", () => {
  it("navigates once after the dwell time in an edge zone", async () => {
    const onNavigate = mock((_direction: -1 | 1) => {});
    const navigator = new EdgeHoverNavigator(onNavigate, 20);

    navigator.setZone("right");
    await Bun.sleep(30);
    navigator.stop();

    expect(onNavigate.mock.calls).toEqual([[1]]);
  });

  it("repeats navigation while the pointer stays in the zone", async () => {
    const onNavigate = mock((_direction: -1 | 1) => {});
    const navigator = new EdgeHoverNavigator(onNavigate, 20);

    navigator.setZone("left");
    await Bun.sleep(50);
    navigator.stop();

    expect(onNavigate.mock.calls.length).toBeGreaterThanOrEqual(2);
    expect(onNavigate).toHaveBeenCalledWith(-1);
  });

  it("does not navigate when the pointer leaves the zone before the dwell elapses", async () => {
    const onNavigate = mock((_direction: -1 | 1) => {});
    const navigator = new EdgeHoverNavigator(onNavigate, 20);

    navigator.setZone("right");
    navigator.setZone(null);
    await Bun.sleep(30);
    navigator.stop();

    expect(onNavigate).not.toHaveBeenCalled();
  });

  it("stop clears a pending dwell", async () => {
    const onNavigate = mock((_direction: -1 | 1) => {});
    const navigator = new EdgeHoverNavigator(onNavigate, 20);

    navigator.setZone("right");
    navigator.stop();
    await Bun.sleep(30);

    expect(onNavigate).not.toHaveBeenCalled();
  });
});
