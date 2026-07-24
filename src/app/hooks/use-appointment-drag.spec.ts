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
  employeeRef: "/v1/contacts/1",
  date: "2026-07-06",
  title: "Projekt Nord",
  color: "bg-primary",
};

type CommandResult<T> =
  | { status: "ok"; data: T }
  | { status: "error"; error: string };
type MoveData =
  | { kind: "moved"; newHref: string }
  | { kind: "sourceDeleteFailed"; newHref: string; sourceHref: string };

const okDeps = () => ({
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
    expect(decideDropAction(payload, target)).toBe("none");
  });

  it("reschedules on the same employee for a different date", () => {
    const target: DropCellTarget = {
      employeeRef: payload.employeeRef,
      date: "2026-07-08",
    };
    expect(decideDropAction(payload, target)).toBe("reschedule");
  });

  it("moves when the target belongs to a different employee", () => {
    const target: DropCellTarget = {
      employeeRef: "/v1/contacts/2",
      date: payload.date,
    };
    expect(decideDropAction(payload, target)).toBe("move");
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
      payload.title,
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
      payload.title,
    );
    expect(deps.updateAssignment).not.toHaveBeenCalled();
  });

  it("surfaces the backend error when the target employee has no calendar", async () => {
    const deps = {
      ...okDeps(),
      moveAssignment: mock(
        async (): Promise<CommandResult<MoveData>> => ({
          status: "error",
          error: "Kein Kalender für diesen Mitarbeiter konfiguriert.",
        }),
      ),
    };
    const outcome = await performDrop(
      payload,
      { employeeRef: "/v1/contacts/2", date: "2026-07-08" },
      deps,
    );

    expect(outcome).toEqual({
      kind: "error",
      message: "Kein Kalender für diesen Mitarbeiter konfiguriert.",
    });
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
  it("navigates once after the dwell time, then stays quiet during the cooldown", async () => {
    const onNavigate = mock((_direction: -1 | 1) => {});
    const navigator = new EdgeHoverNavigator(onNavigate, 20, 20);

    navigator.setZone("right");
    await Bun.sleep(35); // past the 20ms dwell, still within the 20ms cooldown
    navigator.stop();

    expect(onNavigate.mock.calls).toEqual([[1]]);
  });

  it("repeats navigation after the cooldown while the pointer stays in the zone", async () => {
    const onNavigate = mock((_direction: -1 | 1) => {});
    const navigator = new EdgeHoverNavigator(onNavigate, 20, 20);

    navigator.setZone("left");
    await Bun.sleep(90); // dwell(20) + cooldown(20) + dwell(20), with margin
    navigator.stop();

    expect(onNavigate.mock.calls.length).toBeGreaterThanOrEqual(2);
    expect(onNavigate).toHaveBeenCalledWith(-1);
  });

  it("does not navigate when the pointer leaves the zone before the dwell elapses", async () => {
    const onNavigate = mock((_direction: -1 | 1) => {});
    const navigator = new EdgeHoverNavigator(onNavigate, 20, 20);

    navigator.setZone("right");
    navigator.setZone(null);
    await Bun.sleep(30);
    navigator.stop();

    expect(onNavigate).not.toHaveBeenCalled();
  });

  it("leaving the zone during the cooldown prevents the next navigation", async () => {
    const onNavigate = mock((_direction: -1 | 1) => {});
    const navigator = new EdgeHoverNavigator(onNavigate, 20, 20);

    navigator.setZone("right");
    await Bun.sleep(25); // past the dwell, now within the cooldown window
    navigator.setZone(null);
    await Bun.sleep(30); // past when a second fire would have landed
    navigator.stop();

    expect(onNavigate.mock.calls).toEqual([[1]]);
  });

  it("stop clears a pending dwell", async () => {
    const onNavigate = mock((_direction: -1 | 1) => {});
    const navigator = new EdgeHoverNavigator(onNavigate, 20, 20);

    navigator.setZone("right");
    navigator.stop();
    await Bun.sleep(30);

    expect(onNavigate).not.toHaveBeenCalled();
  });

  it("stop clears a pending cooldown", async () => {
    const onNavigate = mock((_direction: -1 | 1) => {});
    const navigator = new EdgeHoverNavigator(onNavigate, 20, 20);

    navigator.setZone("right");
    await Bun.sleep(25); // fires once, now within the cooldown window
    navigator.stop();
    await Bun.sleep(30);

    expect(onNavigate.mock.calls).toEqual([[1]]);
  });
});
