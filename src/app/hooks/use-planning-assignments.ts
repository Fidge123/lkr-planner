import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { CalendarCellEvent, ResolvedProject } from "../../generated/tauri";
import { commands } from "../../generated/tauri";
import { toLocalISODate } from "../util";
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

export interface PlanningAssignmentsState {
  eventsByEmployee: EmployeeEvents;
  errorsByEmployee: EmployeeErrors;
  isLoading: boolean;
  errorMessage: string | null;
  loadedWeekStart: string | null;
  /** Loads the displayed week and its projects again, bypassing both caches. */
  reloadAssignments: () => void;
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
  const cache = useRef<Record<string, WeekData>>({});
  const requestIdRef = useRef(0);
  // The week asked for last, which differs from `loadedWeekStart` while its data loads.
  const requestedWeekRef = useRef<string | null>(null);
  const editsRef = useRef(createEditSequence());
  const [eventsByEmployee, setEventsByEmployee] = useState<EmployeeEvents>({});
  const [errorsByEmployee, setErrorsByEmployee] = useState<EmployeeErrors>({});
  const [isLoading, setIsLoading] = useState(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  // Set in the same batch as the events, so consumers can tell which week's data is rendered mid-fetch.
  const [loadedWeekStart, setLoadedWeekStart] = useState<string | null>(null);

  const show = useCallback((ws: string, data: WeekData) => {
    setEventsByEmployee(data.eventsByEmployee);
    setErrorsByEmployee(data.errorsByEmployee);
    setLoadedWeekStart(ws);
  }, []);

  const updateWeek = useCallback(
    (ws: string, transform: (week: WeekData) => WeekData) => {
      const current = cache.current[ws];
      if (!current) return;
      const next = transform(current);
      if (next === current) return;
      cache.current[ws] = next;
      if (requestedWeekRef.current === ws) {
        show(ws, next);
      }
    },
    [show],
  );

  const resolveProjects = useCallback(
    async (ws: string, forceRefresh = false) => {
      const week = cache.current[ws];
      if (!week) return;
      const refs = projectRefsOf(week);
      if (refs.length === 0) return;

      let resolved: Partial<Record<string, ResolvedProject | null>> | null =
        null;
      try {
        const result = await commands.resolveWeekProjects(refs, forceRefresh);
        if (result.status === "ok") resolved = result.data;
      } catch {}

      updateWeek(ws, (current) =>
        resolved
          ? applyResolvedProjects(current, resolved)
          : settlePendingProjects(current, refs),
      );
    },
    [updateWeek],
  );

  const loadActiveWeek = useCallback(
    async (ws: string, invalidate = false) => {
      if (invalidate) {
        delete cache.current[ws];
      }

      const id = ++requestIdRef.current;
      requestedWeekRef.current = ws;

      const cached = cache.current[ws];
      if (cached) {
        show(ws, cached);
        setIsLoading(false);
        setErrorMessage(null);
        void resolveProjects(ws);
        return;
      }

      setIsLoading(true);
      try {
        const result = await commands.loadWeekEvents(ws, null, "display");
        if (result.status === "error") {
          if (id !== requestIdRef.current) return;
          setErrorMessage(result.error);
          setEventsByEmployee({});
          setErrorsByEmployee({});
          setLoadedWeekStart(ws);
          return;
        }
        // Cache unconditionally, even for a superseded request: it's still valid data for that week.
        // Only the display update below is staleness-gated.
        cache.current[ws] = groupResults(result.data);
        void resolveProjects(ws, invalidate);
        if (id !== requestIdRef.current) return;
        show(ws, cache.current[ws]);
        setErrorMessage(null);
      } catch (error) {
        if (id !== requestIdRef.current) return;
        setErrorMessage(
          error instanceof Error
            ? error.message
            : "Die Einsätze konnten nicht geladen werden.",
        );
        setEventsByEmployee({});
        setErrorsByEmployee({});
        setLoadedWeekStart(ws);
      } finally {
        if (id === requestIdRef.current) {
          setIsLoading(false);
        }
      }
    },
    [show, resolveProjects],
  );

  const prefetchWeek = useCallback(
    async (ws: string) => {
      if (cache.current[ws]) return;
      try {
        const result = await commands.loadWeekEvents(ws, null, "prefetch");
        if (result.status === "ok") {
          cache.current[ws] = groupResults(result.data);
          void resolveProjects(ws);
        }
      } catch {}
    },
    [resolveProjects],
  );

  useEffect(() => {
    if (debouncedWeekStart === null) return;
    let cancelled = false;
    let idleTimer: ReturnType<typeof setTimeout> | undefined;
    let settleIdle: (() => void) | undefined;
    void loadWeekWithPrefetch({
      weekStart: debouncedWeekStart,
      loadActive: loadActiveWeek,
      prefetch: prefetchWeek,
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
  }, [debouncedWeekStart, loadActiveWeek, prefetchWeek]);

  const reloadAssignments = useCallback(() => {
    if (weekStart === null) return;
    void loadActiveWeek(weekStart, true);
  }, [weekStart, loadActiveWeek]);

  // A failed refresh reloads the week, or drops it when no longer shown, so optimistic state never outlives an unknown server state.
  const refreshEmployees = useCallback(
    async (ws: string, employeeRefs: string[]) => {
      if (employeeRefs.length === 0) return;
      const isCurrent = editsRef.current.requestRefresh();
      const discard = () => {
        if (requestedWeekRef.current === ws) {
          void loadActiveWeek(ws, true);
        } else {
          delete cache.current[ws];
        }
      };
      try {
        const result = await commands.loadWeekEvents(
          ws,
          employeeRefs,
          "refresh",
        );
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
    },
    [loadActiveWeek, resolveProjects, updateWeek],
  );

  const removeEvent = useCallback(
    (uid: string) => {
      editsRef.current.markEdit();
      for (const ws of Object.keys(cache.current)) {
        updateWeek(ws, (week) => removeEventFromWeek(week, uid));
      }
    },
    [updateWeek],
  );

  // A drag may end in another week, so the source week's cache, still holding the old slot, is dropped.
  const invalidateWeeksContaining = useCallback(
    (uid: string, keep: string | null = null) => {
      for (const [ws, data] of Object.entries(cache.current)) {
        if (ws !== keep && findEvent(data, uid)) {
          delete cache.current[ws];
        }
      }
    },
    [],
  );

  const moveEvent = useCallback(
    (uid: string, target: CellTarget) => {
      const ws = requestedWeekRef.current;
      if (ws === null) return null;
      const event = Object.values(cache.current)
        .map((week) => findEvent(week, uid))
        .find((found) => found !== null);
      editsRef.current.markEdit();
      invalidateWeeksContaining(uid, ws);
      if (event) {
        updateWeek(ws, (week) => relocateEvent(week, event, target));
      }
      return ws;
    },
    [invalidateWeeksContaining, updateWeek],
  );

  const getCachedWeek = useCallback(
    (ws: string) => cache.current[ws] ?? null,
    [],
  );

  return useMemo(
    () => ({
      eventsByEmployee,
      errorsByEmployee,
      isLoading,
      errorMessage,
      loadedWeekStart,
      reloadAssignments,
      refreshEmployees: (ws: string, employeeRefs: string[]) => {
        void refreshEmployees(ws, employeeRefs);
      },
      removeEvent,
      moveEvent,
      invalidateWeeksContaining: (uid: string) =>
        invalidateWeeksContaining(uid),
      getCachedWeek,
    }),
    [
      eventsByEmployee,
      errorsByEmployee,
      isLoading,
      errorMessage,
      loadedWeekStart,
      reloadAssignments,
      refreshEmployees,
      removeEvent,
      moveEvent,
      invalidateWeeksContaining,
      getCachedWeek,
    ],
  );
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
