import { afterEach, beforeEach, describe, expect, it, jest } from "bun:test";
import { onlineManager, QueryObserver } from "@tanstack/react-query";
import type { CalendarCellEvent, EmployeeWeekEvents } from "../generated/tauri";
import { createQueryClient } from "./query-client";
import { createWeekCache, lastLoadedAt } from "./week-cache";
import type { WeekData } from "./week-data";

const shownWeek = "2026-08-10";
const otherWeek = "2026-08-17";
const nextDay = "2026-08-11";

type Rows = EmployeeWeekEvents[];

function assignment(
  uid: string,
  overrides: Partial<CalendarCellEvent> = {},
): CalendarCellEvent {
  return {
    uid,
    kind: "assignment",
    title: "Projekt Nord",
    projectStatus: null,
    projectCategory: null,
    date: shownWeek,
    startTime: null,
    endTime: null,
    href: null,
    projectRef: null,
    orderIndex: 0,
    projectPending: false,
    ...overrides,
  };
}

function row(employeeReference: string, ...events: CalendarCellEvent[]) {
  return { employeeReference, events, error: null };
}

function unreachable(): Rows {
  throw new Error("Kalender nicht erreichbar.");
}

function deferred() {
  let resolve: (rows: Rows) => void = () => {};
  const promise = new Promise<Rows>((resolvePromise) => {
    resolve = resolvePromise;
  });
  return { promise, resolve };
}

const settle = async () => {
  for (let i = 0; i < 20; i++) await Promise.resolve();
};

let cleanups: (() => void)[] = [];

beforeEach(() => {
  jest.useFakeTimers({ now: 1_000 });
});

afterEach(() => {
  for (const cleanup of cleanups) cleanup();
  cleanups = [];
  onlineManager.setOnline(true);
  jest.useRealTimers();
});

function setup() {
  const client = createQueryClient();
  const requests: string[] = [];
  const forcedResolutions: boolean[] = [];
  let respond: (weekStart: string) => Rows | Promise<Rows> = () => [
    row("emp-1", assignment("uid-1")),
  ];
  const cache = createWeekCache(client, {
    loadWeekEvents: async (weekStart) => {
      requests.push(weekStart);
      return respond(weekStart);
    },
    resolveWeekProjects: async (_refs, forceRefresh) => {
      forcedResolutions.push(forceRefresh);
      return {};
    },
  });
  cleanups.push(() => client.clear());

  return {
    client,
    cache,
    requests,
    forcedResolutions,
    respond: (next: typeof respond) => {
      respond = next;
    },
    /** Subscribes like the planning grid does, which is what marks a week as shown. */
    show: (weekStart: string) => {
      const observer = new QueryObserver(client, cache.weekQuery(weekStart));
      const unsubscribe = observer.subscribe(() => {});
      cleanups.push(unsubscribe);
      return unsubscribe;
    },
    load: (weekStart: string) => client.fetchQuery(cache.weekQuery(weekStart)),
    week: (weekStart = shownWeek) =>
      client.getQueryData<WeekData>(["week", weekStart]),
    state: (weekStart = shownWeek) => {
      const state = client.getQueryState(["week", weekStart]);
      if (!state) throw new Error(`Nothing was cached for ${weekStart}`);
      return state;
    },
  };
}

describe("week load time", () => {
  it("records when the fetch of the week completed", async () => {
    const { load, respond, state } = setup();
    respond(() => {
      jest.advanceTimersByTime(4_000);
      return [row("emp-1", assignment("uid-1"))];
    });

    await load(shownWeek);

    expect(lastLoadedAt(state())).toBe(5_000);
  });

  it("keeps the fetch time of a cached week when it is shown", async () => {
    const { cache, requests, show, state } = setup();
    await cache.prefetch(otherWeek);
    jest.advanceTimersByTime(180_000);

    show(otherWeek);
    await settle();

    expect(requests).toEqual([otherWeek]);
    expect(lastLoadedAt(state(otherWeek))).toBe(1_000);
  });
});

describe("automatic refresh", () => {
  it("discards every other cached week", async () => {
    const { cache, load, requests } = setup();
    await load(shownWeek);
    await cache.prefetch(otherWeek);

    await cache.refresh(shownWeek);
    await load(otherWeek);

    expect(requests).toEqual([shownWeek, otherWeek, shownWeek, otherWeek]);
  });

  it("keeps the shown week editable while a failed refresh waits for its retry", async () => {
    const { cache, load, respond, week } = setup();
    await load(shownWeek);
    respond(unreachable);
    void cache.refresh(shownWeek);
    await settle();

    cache.moveEvent(shownWeek, "uid-1", {
      employeeRef: "emp-1",
      date: nextDay,
      orderIndex: 0,
    });

    expect(week()?.eventsByEmployee["emp-1"][0].date).toBe(nextDay);
  });

  it("keeps an edit made while the refresh ran over the data it fetched before", async () => {
    const { cache, load, respond, state, week } = setup();
    await load(shownWeek);
    const pending = deferred();
    respond(() => pending.promise);

    const refreshing = cache.refresh(shownWeek);
    cache.moveEvent(shownWeek, "uid-1", {
      employeeRef: "emp-1",
      date: nextDay,
      orderIndex: 0,
    });
    jest.advanceTimersByTime(2_000);
    pending.resolve([row("emp-1", assignment("uid-1"))]);
    await refreshing;

    expect(week()?.eventsByEmployee["emp-1"][0].date).toBe(nextDay);
    expect(lastLoadedAt(state())).toBe(3_000);
  });
});

describe("retry of a failed automatic refresh", () => {
  async function failedRefresh() {
    const setupResult = setup();
    await setupResult.load(shownWeek);
    setupResult.respond(unreachable);
    void setupResult.cache.refresh(shownWeek);
    await settle();
    return setupResult;
  }

  it("holds the error and the load time back and retries 30 seconds later", async () => {
    const { requests, respond, state, week } = await failedRefresh();

    expect(state().error).toBeNull();
    expect(lastLoadedAt(state())).toBe(1_000);
    jest.advanceTimersByTime(29_999);
    await settle();
    expect(requests).toHaveLength(2);

    respond(() => [row("emp-1", assignment("uid-2"))]);
    jest.advanceTimersByTime(1);
    await settle();

    expect(requests).toHaveLength(3);
    expect(week()?.eventsByEmployee["emp-1"].map((e) => e.uid)).toEqual([
      "uid-2",
    ]);
    expect(state().error).toBeNull();
    expect(lastLoadedAt(state())).toBe(31_000);
  });

  it("shows the error with the time of the failed retry and keeps the data", async () => {
    const { requests, state, week } = await failedRefresh();

    jest.advanceTimersByTime(30_000);
    await settle();

    expect(state().error?.message).toBe("Kalender nicht erreichbar.");
    expect(lastLoadedAt(state())).toBe(31_000);
    expect(Object.keys(week()?.eventsByEmployee ?? {})).toEqual(["emp-1"]);
    expect(requests).toHaveLength(3);
  });

  it("drops the pending retry when another week is shown", async () => {
    const { cache, load, requests, respond, show } = setup();
    const leave = show(shownWeek);
    await settle();
    respond(unreachable);
    void cache.refresh(shownWeek);
    await settle();

    leave();
    respond(() => [row("emp-1", assignment("uid-1", { date: otherWeek }))]);
    await load(otherWeek);
    jest.advanceTimersByTime(30_000);
    await settle();

    expect(requests).toEqual([shownWeek, shownWeek, otherWeek]);
  });

  it("skips the retry while offline", async () => {
    const { requests, state } = await failedRefresh();

    onlineManager.setOnline(false);
    jest.advanceTimersByTime(30_000);
    await settle();

    expect(requests).toHaveLength(2);
    expect(state().error).toBeNull();
  });
});

describe("row refresh after a write", () => {
  it("replaces the refreshed rows without restarting the refresh interval", async () => {
    const { cache, load, respond, state, week } = setup();
    await load(shownWeek);
    jest.advanceTimersByTime(60_000);
    respond(() => [row("emp-1", assignment("uid-2"))]);

    await cache.refreshEmployees(shownWeek, ["emp-1"]);

    expect(week()?.eventsByEmployee["emp-1"].map((e) => e.uid)).toEqual([
      "uid-2",
    ]);
    expect(lastLoadedAt(state())).toBe(1_000);
  });

  it("discards a row refresh that an edit overtook", async () => {
    const { cache, load, respond, week } = setup();
    await load(shownWeek);
    const pending = deferred();
    respond(() => pending.promise);

    const refreshing = cache.refreshEmployees(shownWeek, ["emp-1"]);
    cache.moveEvent(shownWeek, "uid-1", {
      employeeRef: "emp-1",
      date: nextDay,
      orderIndex: 0,
    });
    pending.resolve([row("emp-1", assignment("uid-1"))]);
    await refreshing;

    expect(week()?.eventsByEmployee["emp-1"][0].date).toBe(nextDay);
  });
});

describe("project details", () => {
  it("reloads invalidated project details from Daylite", async () => {
    const { cache, client, forcedResolutions } = setup();
    const refs = ["/v1/projects/1"];
    await client.fetchQuery(cache.projectsQuery(refs));

    await client.invalidateQueries({ queryKey: ["projects"] });
    await client.fetchQuery(cache.projectsQuery(refs));

    expect(forcedResolutions).toEqual([false, true]);
  });
});
