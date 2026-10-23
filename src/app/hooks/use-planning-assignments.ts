import { useCallback, useEffect, useMemo, useState } from "react";
import type {
  CalendarCellEvent,
  ResolvedProject,
  WeekLoadPurpose,
} from "../../generated/tauri";
import { commands } from "../../generated/tauri";
import { scheduleTimeout, toLocalISODate } from "../util";
import {
  applyResolvedProjects,
  type CellTarget,
  createEditSequence,
  findEvent,
  groupResults,
  mergeEmployeeRows,
  projectRefsOf,
  relocateEvent,
  removeEvent as removeEventFromWeek,
  settlePendingProjects,
  type WeekData,
} from "../week-data";
import { useLeadingDebounce } from "./use-leading-debounce";

type EmployeeEvents = Record<string, CalendarCellEvent[]>;
type EmployeeErrors = Record<string, string>;

export type { WeekData };

export interface PlanningAssignmentsView {
  eventsByEmployee: EmployeeEvents;
  errorsByEmployee: EmployeeErrors;
  isLoading: boolean;
  errorMessage: string | null;
  /** The week the shown events belong to, which lags the requested week during a fetch. */
  loadedWeekStart: string | null;
  /** When the shown week's data was fetched, or when its load failed. */
  lastLoadedAt: number | null;
}

export interface PlanningAssignmentsState extends PlanningAssignmentsView {
  /** Loads the displayed week and its projects again, bypassing both caches. */
  reloadAssignments: () => void;
  /** For automatic refreshes: keeps the shown data on failure and retries once before showing an error. */
  refreshAssignments: () => void;
  /** Replaces the rows of these employees in that week, without the loading state. */
  refreshEmployees: (weekStart: string, employeeRefs: string[]) => void;
  removeEvent: (uid: string) => void;
  /** Shows the event at the drop target before the write lands, returning the week it edited. */
  moveEvent: (uid: string, target: CellTarget) => string | null;
  invalidateWeeksContaining: (uid: string) => void;
  getCachedWeek: (weekStart: string) => WeekData | null;
}

/** A null week start holds the first load back until the week is known. */
export function usePlanningAssignments(
  weekStart: string | null,
): PlanningAssignmentsState {
  // The debounce absorbs rapid navigation; the first known week must not wait it out.
  const debouncedWeekStart = useLeadingDebounce(weekStart, 200) ?? weekStart;
  const [view, setView] = useState(initialView);
  const [loader] = useState(() =>
    createWeekLoader({
      backend: commands,
      now: Date.now,
      schedule: scheduleTimeout,
      isOnline: () => navigator.onLine,
      onChange: setView,
    }),
  );

  useEffect(() => () => loader.dispose(), [loader]);

  useEffect(() => {
    if (debouncedWeekStart === null) return;
    let cancelled = false;
    let idleTimer: ReturnType<typeof setTimeout> | undefined;
    let settleIdle: (() => void) | undefined;
    void loadWeekWithPrefetch({
      weekStart: debouncedWeekStart,
      loadActive: loader.loadActiveWeek,
      prefetch: loader.prefetchWeek,
      isCancelled: () => cancelled,
      wait: (ms) =>
        new Promise((resolve) => {
          settleIdle = resolve;
          idleTimer = setTimeout(resolve, ms);
        }),
    });
    return () => {
      cancelled = true;
      clearTimeout(idleTimer);
      // Clearing the timer alone leaves the sequence suspended on a promise nothing resolves.
      settleIdle?.();
    };
  }, [debouncedWeekStart, loader]);

  const reloadAssignments = useCallback(() => {
    if (weekStart === null) return;
    void loader.loadActiveWeek(weekStart, "reload");
  }, [weekStart, loader]);

  return useMemo(
    () => ({
      ...view,
      reloadAssignments,
      refreshAssignments: () => {
        void loader.refresh();
      },
      refreshEmployees: (ws: string, employeeRefs: string[]) => {
        void loader.refreshEmployees(ws, employeeRefs);
      },
      removeEvent: loader.removeEvent,
      moveEvent: loader.moveEvent,
      invalidateWeeksContaining: (uid: string) =>
        loader.invalidateWeeksContaining(uid),
      getCachedWeek: loader.getCachedWeek,
    }),
    [view, reloadAssignments, loader],
  );
}

const initialView: PlanningAssignmentsView = {
  eventsByEmployee: {},
  errorsByEmployee: {},
  isLoading: true,
  errorMessage: null,
  loadedWeekStart: null,
  lastLoadedAt: null,
};

const emptyWeek: WeekData = { eventsByEmployee: {}, errorsByEmployee: {} };

interface CachedWeek {
  data: WeekData;
  fetchedAt: number;
}

type LoadMode = "show" | "reload" | "refresh" | "retry";

const refreshRetryDelayMs = 30_000;

type FetchResult =
  | { status: "ok"; data: WeekData }
  | { status: "error"; error: string };

export interface WeekLoaderDeps {
  backend: Pick<typeof commands, "loadWeekEvents" | "resolveWeekProjects">;
  now: () => number;
  schedule: (callback: () => void, ms: number) => () => void;
  isOnline: () => boolean;
  onChange: (view: PlanningAssignmentsView) => void;
}

export function createWeekLoader({
  backend,
  now,
  schedule,
  isOnline,
  onChange,
}: WeekLoaderDeps) {
  const cache = new Map<string, CachedWeek>();
  let requestId = 0;
  let requestedWeek: string | null = null;
  const edits = createEditSequence();
  let view = initialView;
  let cancelRetry = () => {};

  const update = (patch: Partial<PlanningAssignmentsView>) => {
    view = { ...view, ...patch };
    onChange(view);
  };

  const shown = (ws: string, data: WeekData) => ({
    eventsByEmployee: data.eventsByEmployee,
    errorsByEmployee: data.errorsByEmployee,
    loadedWeekStart: ws,
  });

  const updateWeek = (ws: string, transform: (week: WeekData) => WeekData) => {
    const current = cache.get(ws);
    if (!current) return;
    const next = transform(current.data);
    if (next === current.data) return;
    cache.set(ws, { ...current, data: next });
    if (requestedWeek === ws) {
      update(shown(ws, next));
    }
  };

  const resolveProjects = async (ws: string, forceRefresh = false) => {
    const week = cache.get(ws);
    if (!week) return;
    const refs = projectRefsOf(week.data);
    if (refs.length === 0) return;

    let resolved: Partial<Record<string, ResolvedProject | null>> | null = null;
    try {
      const result = await backend.resolveWeekProjects(refs, forceRefresh);
      if (result.status === "ok") resolved = result.data;
    } catch {}

    updateWeek(ws, (current) =>
      resolved
        ? applyResolvedProjects(current, resolved)
        : settlePendingProjects(current, refs),
    );
  };

  const fetchWeek = async (
    ws: string,
    purpose: WeekLoadPurpose,
  ): Promise<FetchResult> => {
    try {
      const result = await backend.loadWeekEvents(ws, null, purpose);
      return result.status === "ok"
        ? { status: "ok", data: groupResults(result.data) }
        : result;
    } catch (error) {
      return {
        status: "error",
        error:
          error instanceof Error
            ? error.message
            : "Die Einsätze konnten nicht geladen werden.",
      };
    }
  };

  const loadActiveWeek = async (ws: string, mode: LoadMode = "show") => {
    if (mode === "reload") {
      cache.delete(ws);
    }

    if (mode !== "retry") {
      cancelRetry();
    }

    const id = ++requestId;
    requestedWeek = ws;
    const isAutomatic = mode === "refresh" || mode === "retry";
    const isUnedited = edits.requestRefresh();

    const cached = cache.get(ws);
    if (cached && !isAutomatic) {
      update({
        ...shown(ws, cached.data),
        lastLoadedAt: cached.fetchedAt,
        isLoading: false,
        errorMessage: null,
      });
      void resolveProjects(ws);
      return;
    }

    update({ isLoading: true });
    const result = await fetchWeek(ws, isAutomatic ? "refresh" : "display");
    const completedAt = now();
    // An edit made meanwhile may postdate this data; its own row refresh follows.
    const keepsEdit = isAutomatic && !isUnedited() && cache.has(ws);
    if (result.status === "ok" && !keepsEdit) {
      // A superseded request still caches its data; only showing it is gated.
      cache.set(ws, { data: result.data, fetchedAt: completedAt });
      void resolveProjects(ws, mode === "reload");
    }
    if (id !== requestId) return;

    // The first attempt often fails on a network still connecting after wake.
    if (result.status === "error" && mode === "refresh") {
      cancelRetry = schedule(() => {
        cancelRetry = () => {};
        if (isOnline()) void loadActiveWeek(ws, "retry");
      }, refreshRetryDelayMs);
      update({ isLoading: false });
      return;
    }
    if (result.status === "error") {
      update({
        ...(isAutomatic ? {} : shown(ws, emptyWeek)),
        errorMessage: result.error,
        lastLoadedAt: completedAt,
        isLoading: false,
      });
      return;
    }
    update({
      ...(keepsEdit ? {} : shown(ws, result.data)),
      errorMessage: null,
      lastLoadedAt: completedAt,
      isLoading: false,
    });
  };

  const refresh = async () => {
    const ws = requestedWeek;
    if (ws === null) return;
    for (const cachedWeek of [...cache.keys()]) {
      if (cachedWeek !== ws) cache.delete(cachedWeek);
    }
    await loadActiveWeek(ws, "refresh");
  };

  const prefetchWeek = async (ws: string) => {
    if (cache.has(ws)) return;
    try {
      const result = await backend.loadWeekEvents(ws, null, "prefetch");
      if (result.status === "ok") {
        cache.set(ws, { data: groupResults(result.data), fetchedAt: now() });
        void resolveProjects(ws);
      }
    } catch {}
  };

  // A failed row refresh must not leave optimistic state on screen.
  const refreshEmployees = async (ws: string, employeeRefs: string[]) => {
    if (employeeRefs.length === 0) return;
    const isCurrent = edits.requestRefresh();
    const discard = () => {
      if (requestedWeek === ws) {
        void loadActiveWeek(ws, "reload");
      } else {
        cache.delete(ws);
      }
    };
    try {
      const result = await backend.loadWeekEvents(ws, employeeRefs, "refresh");
      if (!isCurrent()) return;
      if (result.status === "error") {
        discard();
        return;
      }
      updateWeek(ws, (week) =>
        mergeEmployeeRows(week, employeeRefs, result.data),
      );
      void resolveProjects(ws);
    } catch {
      if (isCurrent()) discard();
    }
  };

  const removeEvent = (uid: string) => {
    edits.markEdit();
    for (const ws of [...cache.keys()]) {
      updateWeek(ws, (week) => removeEventFromWeek(week, uid));
    }
  };

  // A drop into another week leaves the source week's cached copy with the event in its old slot.
  const invalidateWeeksContaining = (
    uid: string,
    keep: string | null = null,
  ) => {
    for (const [ws, week] of cache) {
      if (ws !== keep && findEvent(week.data, uid)) {
        cache.delete(ws);
      }
    }
  };

  const moveEvent = (uid: string, target: CellTarget) => {
    const ws = requestedWeek;
    if (ws === null) return null;
    const event = [...cache.values()]
      .map((week) => findEvent(week.data, uid))
      .find((found) => found !== null);
    edits.markEdit();
    invalidateWeeksContaining(uid, ws);
    if (event) {
      updateWeek(ws, (week) => relocateEvent(week, event, target));
    }
    return ws;
  };

  const getCachedWeek = (ws: string) => cache.get(ws)?.data ?? null;

  return {
    loadActiveWeek,
    prefetchWeek,
    refresh,
    refreshEmployees,
    removeEvent,
    moveEvent,
    invalidateWeeksContaining,
    getCachedWeek,
    dispose: () => cancelRetry(),
  };
}

export interface WeekLoadSequence {
  weekStart: string;
  loadActive: (weekStart: string) => Promise<void>;
  prefetch: (weekStart: string) => Promise<void>;
  isCancelled: () => boolean;
  wait: (ms: number) => Promise<void>;
}

const prefetchIdleMs = 400;

/**
 * A prefetch competes with the active week for CalDAV and Daylite requests.
 * A started week load cannot be cancelled, so it waits for the user to settle first.
 */
export async function loadWeekWithPrefetch({
  weekStart,
  loadActive,
  prefetch,
  isCancelled,
  wait,
}: WeekLoadSequence): Promise<void> {
  await loadActive(weekStart);
  if (isCancelled()) return;

  await wait(prefetchIdleMs);
  if (isCancelled()) return;

  await Promise.all([
    prefetch(adjacentWeek(weekStart, -7)),
    prefetch(adjacentWeek(weekStart, 7)),
  ]);
}

function adjacentWeek(weekStart: string, offsetDays: number): string {
  const [y, m, d] = weekStart.split("-").map(Number);
  return toLocalISODate(new Date(y, m - 1, d + offsetDays));
}
