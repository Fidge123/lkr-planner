import { describe, expect, it, mock } from "bun:test";
import type {
  AppointmentDragPayload,
  CardRect,
  DropCellTarget,
  DropZoneData,
} from "./use-appointment-drag";
import {
  commandDropDeps,
  computeEdgeZone,
  decideDropAction,
  EdgeHoverNavigator,
  insertionIndexAt,
  performDrop,
  resolveDropTarget,
} from "./use-appointment-drag";

const updateAssignmentCommand = mock(
  async (): Promise<CommandResult<null>> => ({ status: "ok", data: null }),
);
mock.module("../../generated/tauri", () => ({
  commands: {
    updateAssignment: updateAssignmentCommand,
    reorderAssignment: mock(),
    moveAssignment: mock(),
  },
}));

const payload: AppointmentDragPayload = {
  uid: "uid-1",
  href: "/calendars/emp-a/uid-1.ics",
  projectRef: "/v1/projects/42",
  employeeRef: "/v1/contacts/1",
  date: "2026-07-06",
  title: "Projekt Nord",
  color: "bg-primary",
  categoryColor: null,
  position: 1,
};

// Three 40px cards stacked from y=100, so their midpoints are 120, 170 and 220.
function stack(uids: [string, string, string]): CardRect[] {
  return uids.map((uid, i) => ({ uid, top: 100 + i * 50, height: 40 }));
}

const stackedCards = stack(["uid-a", "uid-b", "uid-c"]);
/** The dragged card (`payload.uid`) sits at `payload.position` among its own cell's cards. */
const ownCellCards = stack(["uid-a", payload.uid, "uid-b"]);
const otherCellCards = stack(["uid-x", "uid-y", "uid-z"]);

function cellZone(
  cards: CardRect[],
  employeeRef = payload.employeeRef,
  date = payload.date,
): DropZoneData {
  return { kind: "cell", employeeRef, date, cardRects: () => cards };
}

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
  reorderAssignment: mock(
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
      orderIndex: payload.position,
    };
    expect(decideDropAction(payload, target)).toBe("none");
  });

  it("reschedules on the same employee for a different date", () => {
    const target: DropCellTarget = {
      employeeRef: payload.employeeRef,
      date: "2026-07-08",
      orderIndex: 3,
    };
    expect(decideDropAction(payload, target)).toBe("reschedule");
  });

  it("moves when the target belongs to a different employee", () => {
    const target: DropCellTarget = {
      employeeRef: "/v1/contacts/2",
      date: payload.date,
      orderIndex: 3,
    };
    expect(decideDropAction(payload, target)).toBe("move");
  });

  it("reorders when the position inside the originating cell changes", () => {
    const target: DropCellTarget = {
      employeeRef: payload.employeeRef,
      date: payload.date,
      orderIndex: 0,
    };
    expect(decideDropAction(payload, target)).toBe("reorder");
  });

  it("is a no-op when the card is dropped back onto its own position", () => {
    const target: DropCellTarget = {
      employeeRef: payload.employeeRef,
      date: payload.date,
      orderIndex: payload.position,
    };
    expect(decideDropAction(payload, target)).toBe("none");
  });
});

describe("performDrop", () => {
  it("makes no persistence call when dropped on the originating cell", async () => {
    const deps = okDeps();
    const outcome = await performDrop(
      payload,
      {
        employeeRef: payload.employeeRef,
        date: payload.date,
        orderIndex: payload.position,
      },
      deps,
    );

    expect(outcome).toEqual({ kind: "none" });
    expect(deps.updateAssignment).not.toHaveBeenCalled();
    expect(deps.moveAssignment).not.toHaveBeenCalled();
    expect(deps.reorderAssignment).not.toHaveBeenCalled();
  });

  it("reschedules via updateAssignment within the same employee", async () => {
    const deps = okDeps();
    const outcome = await performDrop(
      payload,
      { employeeRef: payload.employeeRef, date: "2026-07-08", orderIndex: 2 },
      deps,
    );

    expect(outcome).toEqual({ kind: "done" });
    expect(deps.updateAssignment).toHaveBeenCalledWith(
      payload.href,
      payload.uid,
      "2026-07-08",
      payload.projectRef,
      payload.title,
      2,
    );
    expect(deps.moveAssignment).not.toHaveBeenCalled();
  });

  it("moves via moveAssignment to a different employee", async () => {
    const deps = okDeps();
    const outcome = await performDrop(
      payload,
      { employeeRef: "/v1/contacts/2", date: "2026-07-08", orderIndex: 0 },
      deps,
    );

    expect(outcome).toEqual({ kind: "done" });
    expect(deps.moveAssignment).toHaveBeenCalledWith(
      payload.href,
      "/v1/contacts/2",
      "2026-07-08",
      payload.projectRef,
      payload.title,
      0,
    );
    expect(deps.updateAssignment).not.toHaveBeenCalled();
  });

  it("reorders within the originating cell without a cross-calendar move", async () => {
    const deps = okDeps();
    const outcome = await performDrop(
      payload,
      {
        employeeRef: payload.employeeRef,
        date: payload.date,
        orderIndex: 0,
      },
      deps,
    );

    expect(outcome).toEqual({ kind: "done" });
    expect(deps.reorderAssignment).toHaveBeenCalledWith(
      payload.href,
      payload.uid,
      payload.date,
      0,
    );
    expect(deps.updateAssignment).not.toHaveBeenCalled();
    expect(deps.moveAssignment).not.toHaveBeenCalled();
  });

  it("surfaces the backend error when a reorder fails", async () => {
    const deps = {
      ...okDeps(),
      reorderAssignment: mock(
        async (): Promise<CommandResult<null>> => ({
          status: "error",
          error: "Kalenderserver antwortete mit HTTP 500",
        }),
      ),
    };
    const outcome = await performDrop(
      payload,
      { employeeRef: payload.employeeRef, date: payload.date, orderIndex: 0 },
      deps,
    );

    expect(outcome).toEqual({
      kind: "error",
      message: "Kalenderserver antwortete mit HTTP 500",
    });
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
      { employeeRef: "/v1/contacts/2", date: "2026-07-08", orderIndex: 0 },
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
      { employeeRef: "/v1/contacts/2", date: "2026-07-08", orderIndex: 0 },
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
      {
        employeeRef: payload.employeeRef,
        date: "2026-07-08",
        orderIndex: 3,
      },
      deps,
    );

    expect(outcome).toEqual({
      kind: "error",
      message: "Kalenderserver antwortete mit HTTP 500",
    });
  });
});

describe("insertionIndexAt", () => {
  it("counts no card as passed above the whole stack", () => {
    expect(insertionIndexAt(stackedCards, 90)).toBe(0);
  });

  it("lands before a card while the pointer is in its upper half", () => {
    expect(insertionIndexAt(stackedCards, 115)).toBe(0);
  });

  it("lands after a card once the pointer passes its midpoint", () => {
    expect(insertionIndexAt(stackedCards, 125)).toBe(1);
  });

  it("appends when the pointer is below every card", () => {
    expect(insertionIndexAt(stackedCards, 400)).toBe(3);
  });

  it("appends on an empty cell", () => {
    expect(insertionIndexAt([], 400)).toBe(0);
  });

  it("is stable once a preview is spliced in at the index it returned", () => {
    // Inserting a 50px preview at index 1 pushes only the cards already below the pointer
    // further down, so re-measuring must yield the same index instead of oscillating.
    const pointerY = 125;
    const index = insertionIndexAt(stackedCards, pointerY);
    const shifted = stackedCards.map((card, i) =>
      i >= index ? { ...card, top: card.top + 50 } : card,
    );

    expect(insertionIndexAt(shifted, pointerY)).toBe(index);
  });
});

describe("resolveDropTarget", () => {
  it("takes the cell and date from the zone under the pointer", () => {
    const target = resolveDropTarget(
      payload,
      cellZone(otherCellCards, "/v1/contacts/2", "2026-07-09"),
      115,
    );

    expect(target.employeeRef).toBe("/v1/contacts/2");
    expect(target.date).toBe("2026-07-09");
  });

  it("places the card before the one whose upper half holds the pointer", () => {
    const target = resolveDropTarget(
      payload,
      cellZone(otherCellCards, "/v1/contacts/2"),
      165,
    );

    expect(target.orderIndex).toBe(1);
  });

  it("places the card after the one whose lower half holds the pointer", () => {
    const target = resolveDropTarget(
      payload,
      cellZone(otherCellCards, "/v1/contacts/2"),
      180,
    );

    expect(target.orderIndex).toBe(2);
  });

  it("appends when the pointer sits below every card in the cell", () => {
    const target = resolveDropTarget(
      payload,
      cellZone(otherCellCards, "/v1/contacts/2"),
      400,
    );

    expect(target.orderIndex).toBe(3);
  });

  it("does not count the dragged card as its own neighbour", () => {
    // The dragged card is one of the three, so a pointer below all of them resolves to 2
    // rather than 3 in its own cell.
    const target = resolveDropTarget(payload, cellZone(ownCellCards), 400);

    expect(target.orderIndex).toBe(2);
  });

  it("resolves to the dragged card's own position when dropped back on itself", () => {
    // Pointer over the dragged card itself, which sits second in its cell.
    const ownCell = resolveDropTarget(payload, cellZone(ownCellCards), 170);

    expect(ownCell.orderIndex).toBe(payload.position);
    expect(decideDropAction(payload, ownCell)).toBe("none");
  });
});

// Composes the whole drop gesture the grid runs on drag end: the cell under the pointer plus
// the cards' geometry become an insertion position, which decides the action and the command.
describe("drop gesture end to end", () => {
  it("reorders a card to the front of its own day", async () => {
    const deps = okDeps();

    // Above every card in the dragged card's own cell.
    const target = resolveDropTarget(payload, cellZone(ownCellCards), 90);
    const outcome = await performDrop(payload, target, deps);

    expect(outcome).toEqual({ kind: "done" });
    expect(deps.reorderAssignment).toHaveBeenCalledWith(
      payload.href,
      payload.uid,
      payload.date,
      0,
    );
    expect(deps.moveAssignment).not.toHaveBeenCalled();
    expect(deps.updateAssignment).not.toHaveBeenCalled();
  });

  it("lands after a specific card of another employee", async () => {
    const deps = okDeps();

    // Lower half of the first card in the target cell.
    const target = resolveDropTarget(
      payload,
      cellZone(otherCellCards, "/v1/contacts/2", "2026-07-08"),
      130,
    );
    const outcome = await performDrop(payload, target, deps);

    expect(outcome).toEqual({ kind: "done" });
    expect(deps.moveAssignment).toHaveBeenCalledWith(
      payload.href,
      "/v1/contacts/2",
      "2026-07-08",
      payload.projectRef,
      payload.title,
      1,
    );
  });

  it("appends when the drop lands on the empty area of another employee's cell", async () => {
    const deps = okDeps();

    const target = resolveDropTarget(
      payload,
      cellZone(otherCellCards, "/v1/contacts/2", "2026-07-08"),
      400,
    );
    const outcome = await performDrop(payload, target, deps);

    expect(outcome).toEqual({ kind: "done" });
    expect(deps.moveAssignment).toHaveBeenCalledWith(
      payload.href,
      "/v1/contacts/2",
      "2026-07-08",
      payload.projectRef,
      payload.title,
      3,
    );
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

describe("commandDropDeps", () => {
  it("rescheduling a drag never overrides the fixed-appointment protection", async () => {
    await commandDropDeps().updateAssignment(
      "/cal/uid-1.ics",
      "uid-1",
      "2026-07-08",
      "/v1/projects/42",
      "Projekt Nord",
      0,
    );

    expect(updateAssignmentCommand).toHaveBeenCalledWith(
      expect.objectContaining({ overrideProtection: false }),
    );
  });
});
