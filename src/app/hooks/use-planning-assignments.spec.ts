import { describe, expect, it } from "bun:test";
import type {
  CalendarCellEvent,
  EmployeeWeekEvents,
} from "../../generated/tauri";
import { manualClock } from "../../test-support/manual-clock";
import {
  createWeekLoader,
  loadWeekWithPrefetch,
  type PlanningAssignmentsView,
  type WeekLoadSequence,
} from "./use-planning-assignments";

function deferred() {
  let resolve: () => void = () => {};
  const promise = new Promise<void>((resolvePromise) => {
    resolve = resolvePromise;
  });
  return { promise, resolve };
}

function sequence(overrides: Partial<WeekLoadSequence> = {}) {
  const prefetched: string[] = [];
  const waited: number[] = [];

  const run = () =>
    loadWeekWithPrefetch({
      weekStart: "2026-08-10",
      loadActive: async () => {},
      prefetch: async (weekStart) => {
        prefetched.push(weekStart);
      },
      isCancelled: () => false,
      wait: async (ms) => {
        waited.push(ms);
      },
      ...overrides,
    });

  return { run, prefetched, waited };
}

describe("loadWeekWithPrefetch", () => {
  it("holds the prefetch back until the active week has loaded", async () => {
    const active = deferred();
    const { run, prefetched } = sequence({ loadActive: () => active.promise });

    const running = run();
    await Promise.resolve();
    expect(prefetched).toEqual([]);

    active.resolve();
    await running;

    expect(prefetched.sort()).toEqual(["2026-08-03", "2026-08-17"]);
  });

  it("waits out the idle window before dispatching the prefetch", async () => {
    const { run, prefetched, waited } = sequence({
      wait: async (ms) => {
        waited.push(ms);
        expect(prefetched).toEqual([]);
      },
    });

    await run();

    expect(waited).toHaveLength(1);
    expect(waited[0]).toBeGreaterThan(0);
    expect(prefetched.sort()).toEqual(["2026-08-03", "2026-08-17"]);
  });

  it("skips the prefetch when the week changed during the idle window", async () => {
    let cancelled = false;
    const { run, prefetched } = sequence({
      isCancelled: () => cancelled,
      wait: async () => {
        cancelled = true;
      },
    });

    await run();

    expect(prefetched).toEqual([]);
  });

  it("skips the idle window entirely when the week already changed", async () => {
    const { run, prefetched, waited } = sequence({ isCancelled: () => true });

    await run();

    expect(waited).toEqual([]);
    expect(prefetched).toEqual([]);
  });

  it("still prefetches when the active week came from the cache", async () => {
    const { run, prefetched } = sequence();

    await run();

    expect(prefetched.sort()).toEqual(["2026-08-03", "2026-08-17"]);
  });
});

const shownWeek = "2026-08-10";
const otherWeek = "2026-08-17";

type LoadResult =
  | { status: "ok"; data: EmployeeWeekEvents[] }
  | { status: "error"; error: string };

function assignment(uid: string, date = shownWeek): CalendarCellEvent {
  return {
    uid,
    kind: "assignment",
    title: "Projekt Nord",
    projectStatus: null,
    projectCategory: null,
    date,
    startTime: null,
    endTime: null,
    href: null,
    projectRef: null,
    orderIndex: 0,
    projectPending: false,
  };
}

function ok(...rows: EmployeeWeekEvents[]): LoadResult {
  return { status: "ok", data: rows };
}

function row(
  employeeReference: string,
  ...events: CalendarCellEvent[]
): EmployeeWeekEvents {
  return { employeeReference, events, error: null };
}

const settle = () => new Promise((resolve) => setTimeout(resolve, 0));

function weekLoader() {
  const clock = manualClock(1_000);
  const requests: string[] = [];
  let respond: (weekStart: string) => LoadResult | Promise<LoadResult> = () =>
    ok(row("emp-1", assignment("uid-1")));
  let view: PlanningAssignmentsView | null = null;
  let online = true;

  const loader = createWeekLoader({
    backend: {
      loadWeekEvents: async (weekStart) => {
        requests.push(weekStart);
        return respond(weekStart);
      },
      resolveWeekProjects: async () => ({ status: "ok", data: {} }),
    },
    now: clock.now,
    schedule: clock.schedule,
    isOnline: () => online,
    onChange: (next) => {
      view = next;
    },
  });

  return {
    clock,
    loader,
    requests,
    view: () => {
      if (view === null) throw new Error("Nothing was shown yet");
      return view;
    },
    respond: (next: typeof respond) => {
      respond = next;
    },
    goOffline: () => {
      online = false;
    },
  };
}

describe("createWeekLoader load time", () => {
  it("records when the fetch of the shown week completed", async () => {
    const { clock, loader, respond, view } = weekLoader();
    respond(() => {
      clock.advance(4_000);
      return ok(row("emp-1", assignment("uid-1")));
    });

    await loader.loadActiveWeek(shownWeek);

    expect(view().lastLoadedAt).toBe(5_000);
    expect(view().loadedWeekStart).toBe(shownWeek);
  });

  it("keeps the fetch time of a cached week when it is shown", async () => {
    const { clock, loader, requests, view } = weekLoader();
    await loader.prefetchWeek(otherWeek);
    clock.advance(180_000);

    await loader.loadActiveWeek(otherWeek);

    expect(requests).toEqual([otherWeek]);
    expect(view().loadedWeekStart).toBe(otherWeek);
    expect(view().lastLoadedAt).toBe(1_000);
  });

  it("records when a failed load completed", async () => {
    const { clock, loader, respond, view } = weekLoader();
    respond(() => {
      clock.advance(2_000);
      return { status: "error", error: "Kalender nicht erreichbar." };
    });

    await loader.loadActiveWeek(shownWeek);

    expect(view().errorMessage).toBe("Kalender nicht erreichbar.");
    expect(view().lastLoadedAt).toBe(3_000);
  });
});

function deferredResult() {
  let resolve: (result: LoadResult) => void = () => {};
  const promise = new Promise<LoadResult>((resolvePromise) => {
    resolve = resolvePromise;
  });
  return { promise, resolve };
}

const nextDay = "2026-08-11";

describe("createWeekLoader automatic refresh", () => {
  it("fetches the shown week again instead of showing it from the cache", async () => {
    const { loader, requests } = weekLoader();
    await loader.loadActiveWeek(shownWeek);

    await loader.refresh();

    expect(requests).toEqual([shownWeek, shownWeek]);
  });

  it("discards every other cached week", async () => {
    const { loader, requests } = weekLoader();
    await loader.loadActiveWeek(shownWeek);
    await loader.prefetchWeek(otherWeek);

    await loader.refresh();
    await loader.loadActiveWeek(otherWeek);

    expect(requests).toEqual([shownWeek, otherWeek, shownWeek, otherWeek]);
  });

  it("keeps showing the loaded week while the refresh runs", async () => {
    const { loader, respond, view } = weekLoader();
    await loader.loadActiveWeek(shownWeek);
    const pending = deferredResult();
    respond(() => pending.promise);

    const refreshing = loader.refresh();

    expect(view().isLoading).toBe(true);
    expect(Object.keys(view().eventsByEmployee)).toEqual(["emp-1"]);
    pending.resolve(ok(row("emp-1", assignment("uid-2"))));
    await refreshing;
    expect(view().isLoading).toBe(false);
    expect(view().eventsByEmployee["emp-1"].map((e) => e.uid)).toEqual([
      "uid-2",
    ]);
  });

  it("keeps the shown events and per-employee errors when the fetch fails", async () => {
    const { loader, respond, view } = weekLoader();
    respond(() =>
      ok(row("emp-1", assignment("uid-1")), {
        employeeReference: "emp-2",
        events: [],
        error: "Kalender nicht gefunden.",
      }),
    );
    await loader.loadActiveWeek(shownWeek);
    const before = view();
    respond(() => ({ status: "error", error: "Kalender nicht erreichbar." }));

    await loader.refresh();

    expect(view().eventsByEmployee).toBe(before.eventsByEmployee);
    expect(view().errorsByEmployee).toEqual({
      "emp-2": "Kalender nicht gefunden.",
    });
    expect(view().loadedWeekStart).toBe(shownWeek);
    expect(view().isLoading).toBe(false);
  });

  it("keeps the shown week editable after a failed refresh", async () => {
    const { loader, respond, view } = weekLoader();
    await loader.loadActiveWeek(shownWeek);
    respond(() => ({ status: "error", error: "Kalender nicht erreichbar." }));
    await loader.refresh();

    loader.moveEvent("uid-1", {
      employeeRef: "emp-1",
      date: nextDay,
      orderIndex: 0,
    });

    expect(view().eventsByEmployee["emp-1"][0].date).toBe(nextDay);
  });

  it("clears the error message once a refresh succeeds", async () => {
    const { loader, respond, view } = weekLoader();
    respond(() => ({ status: "error", error: "Kalender nicht erreichbar." }));
    await loader.loadActiveWeek(shownWeek);
    respond(() => ok(row("emp-1", assignment("uid-1"))));

    await loader.refresh();

    expect(view().errorMessage).toBeNull();
    expect(Object.keys(view().eventsByEmployee)).toEqual(["emp-1"]);
  });

  it("keeps an edit made while the refresh ran over the data it fetched before", async () => {
    const { clock, loader, respond, view } = weekLoader();
    await loader.loadActiveWeek(shownWeek);
    const pending = deferredResult();
    respond(() => pending.promise);

    const refreshing = loader.refresh();
    loader.moveEvent("uid-1", {
      employeeRef: "emp-1",
      date: nextDay,
      orderIndex: 0,
    });
    clock.advance(2_000);
    pending.resolve(ok(row("emp-1", assignment("uid-1"))));
    await refreshing;

    expect(view().eventsByEmployee["emp-1"][0].date).toBe(nextDay);
    expect(view().lastLoadedAt).toBe(3_000);
  });

  it("empties the grid behind the error when a manual reload fails", async () => {
    const { loader, respond, view } = weekLoader();
    await loader.loadActiveWeek(shownWeek);
    respond(() => ({ status: "error", error: "Kalender nicht erreichbar." }));

    await loader.loadActiveWeek(shownWeek, "reload");

    expect(view().eventsByEmployee).toEqual({});
    expect(view().errorMessage).toBe("Kalender nicht erreichbar.");
  });
});

describe("createWeekLoader retry of a failed automatic refresh", () => {
  const unreachable = (): LoadResult => ({
    status: "error",
    error: "Kalender nicht erreichbar.",
  });

  async function failedRefresh() {
    const setup = weekLoader();
    await setup.loader.loadActiveWeek(shownWeek);
    setup.respond(unreachable);
    await setup.loader.refresh();
    return setup;
  }

  it("holds the error and the load time back and retries 30 seconds later", async () => {
    const { clock, requests, respond, view } = await failedRefresh();

    expect(view().errorMessage).toBeNull();
    expect(view().lastLoadedAt).toBe(1_000);
    clock.advance(29_999);
    expect(requests).toHaveLength(2);

    respond(() => ok(row("emp-1", assignment("uid-2"))));
    clock.advance(1);
    await settle();

    expect(requests).toHaveLength(3);
    expect(view().eventsByEmployee["emp-1"].map((e) => e.uid)).toEqual([
      "uid-2",
    ]);
    expect(view().errorMessage).toBeNull();
    expect(view().lastLoadedAt).toBe(31_000);
  });

  it("shows the error with the time of the failed retry and retries only once", async () => {
    const { clock, requests, view } = await failedRefresh();

    clock.advance(30_000);
    await settle();

    expect(view().errorMessage).toBe("Kalender nicht erreichbar.");
    expect(view().lastLoadedAt).toBe(31_000);
    expect(Object.keys(view().eventsByEmployee)).toEqual(["emp-1"]);
    clock.advance(600_000);
    await settle();
    expect(requests).toHaveLength(3);
  });

  it("drops the pending retry when the shown week is reloaded", async () => {
    const { clock, loader, requests, respond } = await failedRefresh();
    respond(() => ok(row("emp-1", assignment("uid-1"))));

    await loader.loadActiveWeek(shownWeek, "reload");
    clock.advance(30_000);
    await settle();

    expect(requests).toHaveLength(3);
  });

  it("drops the pending retry when another week is shown", async () => {
    const { clock, loader, requests, respond } = await failedRefresh();
    respond(() => ok(row("emp-1", assignment("uid-1", otherWeek))));

    await loader.loadActiveWeek(otherWeek);
    clock.advance(30_000);
    await settle();

    expect(requests).toEqual([shownWeek, shownWeek, otherWeek]);
  });

  it("skips the retry while offline", async () => {
    const { clock, goOffline, requests, view } = await failedRefresh();

    goOffline();
    clock.advance(30_000);
    await settle();

    expect(requests).toHaveLength(2);
    expect(view().errorMessage).toBeNull();
  });

  it("cancels the pending retry when disposed", async () => {
    const { clock, loader } = await failedRefresh();

    loader.dispose();

    expect(clock.pendingTimers()).toBe(0);
  });
});
