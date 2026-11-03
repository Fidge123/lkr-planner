import {
  type QueryClient,
  queryOptions,
  replaceEqualDeep,
} from "@tanstack/react-query";
import type { ResolvedProject, WeekLoadPurpose } from "../generated/tauri";
import type {
  loadWeekEvents,
  resolveWeekProjects,
} from "../services/assignments";
import { toLocalISODate } from "./util";
import {
  applyResolvedProjects,
  type CellTarget,
  createEditSequence,
  findEvent,
  groupResults,
  mergeEmployeeRows,
  projectRefsOf,
  relocateEvent,
  removeEvent,
  settlePendingProjects,
  type WeekData,
} from "./week-data";

interface WeekBackend {
  loadWeekEvents: typeof loadWeekEvents;
  resolveWeekProjects: typeof resolveWeekProjects;
}

type ResolvedProjects = Partial<Record<string, ResolvedProject | null>>;

export type WeekCache = ReturnType<typeof createWeekCache>;

const refreshRetryDelayMs = 30_000;

const weekKey = (weekStart: string) => ["week", weekStart];
const projectsKey = (refs: string[]) => ["projects", refs];

/** A failed load counts as completed, so a failing server is not asked again at once. */
export const lastLoadedAt = (state: {
  dataUpdatedAt: number;
  errorUpdatedAt: number;
}) => Math.max(state.dataUpdatedAt, state.errorUpdatedAt) || null;

export const sortedProjectRefs = (week: WeekData) => projectRefsOf(week).sort();

/** A project that failed to resolve stops being pending but stays unresolved. */
export function joinProjects(
  week: WeekData,
  refs: string[],
  projects: ResolvedProjects | undefined,
  hasFailed: boolean,
): WeekData {
  if (projects) return applyResolvedProjects(week, projects);
  return hasFailed ? settlePendingProjects(week, refs) : week;
}

/** Week events and project details are cached apart and joined when read. */
export function createWeekCache(client: QueryClient, backend: WeekBackend) {
  const edits = createEditSequence();
  // The grid puts its own options back on every render, so a refresh and its retry are marked here instead.
  const refreshing = new Set<string>();

  // A refresh that an optimistic edit overtook keeps the edited week; the edit's row refresh follows.
  const fetchWeek = async (weekStart: string, purpose: WeekLoadPurpose) => {
    const isUnedited = edits.requestRefresh();
    const fetched = groupResults(
      await backend.loadWeekEvents(weekStart, null, purpose),
    );
    const current = client.getQueryData<WeekData>(weekKey(weekStart));
    const keepsEdit = purpose === "refresh" && !isUnedited() && current;
    return keepsEdit ? current : fetched;
  };

  const weekQuery = (weekStart: string) =>
    queryOptions({
      queryKey: weekKey(weekStart),
      queryFn: () =>
        fetchWeek(weekStart, refreshing.has(weekStart) ? "refresh" : "display"),
    });

  const projectsQuery = (refs: string[]) =>
    queryOptions({
      queryKey: projectsKey(refs),
      // An invalidated entry is reloaded from Daylite instead of the backend's project cache.
      // A failed fetch marks its entry invalidated too, so only one that last succeeded is a reload.
      queryFn: () => {
        const state = client.getQueryState(projectsKey(refs));
        const isReload =
          state !== undefined &&
          state.isInvalidated &&
          state.errorUpdatedAt < state.dataUpdatedAt;
        return backend.resolveWeekProjects(refs, isReload);
      },
      enabled: refs.length > 0,
    });

  const isShown = (weekStart: string) =>
    (client
      .getQueryCache()
      .find({ queryKey: weekKey(weekStart) })
      ?.getObserversCount() ?? 0) > 0;

  // setQueryData would count as a load, which restarts the refresh interval and clears an invalidation or error.
  const updateWeek = (
    weekStart: string,
    transform: (week: WeekData) => WeekData,
  ) => {
    const query = client
      .getQueryCache()
      .find<WeekData>({ queryKey: weekKey(weekStart) });
    const week = query?.state.data;
    if (!query || !week) return;
    const next = replaceEqualDeep(week, transform(week));
    if (next !== week) query.setState({ data: next });
  };

  const cachedWeeks = () =>
    client
      .getQueryCache()
      .findAll({ queryKey: ["week"] })
      .map((query) => ({
        weekStart: query.queryKey[1] as string,
        data: query.state.data as WeekData | undefined,
        isShown: query.getObserversCount() > 0,
      }));

  const reload = (weekStart: string) => {
    const week = client.getQueryData<WeekData>(weekKey(weekStart));
    if (week) {
      void client.invalidateQueries({
        queryKey: projectsKey(sortedProjectRefs(week)),
        exact: true,
      });
    }
    return client.invalidateQueries({
      queryKey: weekKey(weekStart),
      exact: true,
    });
  };

  /** A drop into another week leaves the source week's cached copy with the event in its old slot. */
  const dropWeeksContaining = (uid: string, keep: string | null = null) => {
    for (const week of cachedWeeks()) {
      if (
        week.weekStart !== keep &&
        !week.isShown &&
        week.data &&
        findEvent(week.data, uid)
      ) {
        client.removeQueries({ queryKey: weekKey(week.weekStart) });
      }
    }
  };

  return {
    weekQuery,
    projectsQuery,
    reload,
    dropWeeksContaining,

    prefetch: async (weekStart: string) => {
      await client.prefetchQuery({
        ...weekQuery(weekStart),
        queryFn: () => fetchWeek(weekStart, "prefetch"),
      });
      const week = client.getQueryData<WeekData>(weekKey(weekStart));
      if (week)
        await client.prefetchQuery(projectsQuery(sortedProjectRefs(week)));
    },

    /** Keeps the shown data while it runs and retries a failure once before reporting it. */
    refresh: async (weekStart: string) => {
      client.removeQueries({
        queryKey: ["week"],
        predicate: (query) => query.queryKey[1] !== weekStart,
      });
      refreshing.add(weekStart);
      const week = await client
        .fetchQuery({
          ...weekQuery(weekStart),
          staleTime: 0,
          retry: 1,
          retryDelay: refreshRetryDelayMs,
        })
        .catch(() => null)
        .finally(() => refreshing.delete(weekStart));
      if (!week) return;
      // Unlike an invalidation, a refetch lets the backend's project cache answer for entries still fresh.
      await client.refetchQueries({
        queryKey: projectsKey(sortedProjectRefs(week)),
        exact: true,
      });
    },

    /** A failed row refresh must not leave optimistic state on screen. */
    refreshEmployees: async (weekStart: string, employeeRefs: string[]) => {
      if (employeeRefs.length === 0) return;
      const isCurrent = edits.requestRefresh();
      try {
        const entries = await backend.loadWeekEvents(
          weekStart,
          employeeRefs,
          "refresh",
        );
        if (!isCurrent()) return;
        updateWeek(weekStart, (week) =>
          mergeEmployeeRows(week, employeeRefs, entries),
        );
      } catch {
        if (!isCurrent()) return;
        if (isShown(weekStart)) {
          void reload(weekStart);
        } else {
          client.removeQueries({ queryKey: weekKey(weekStart) });
        }
      }
    },

    removeEvent: (uid: string) => {
      edits.markEdit();
      for (const week of cachedWeeks()) {
        updateWeek(week.weekStart, (data) => removeEvent(data, uid));
      }
    },

    /** Shows the event at the drop target before the write lands. */
    moveEvent: (weekStart: string, uid: string, target: CellTarget) => {
      const event = cachedWeeks()
        .map((week) => (week.data ? findEvent(week.data, uid) : null))
        .find((found) => found !== null);
      edits.markEdit();
      dropWeeksContaining(uid, weekStart);
      if (event) {
        updateWeek(weekStart, (week) => relocateEvent(week, event, target));
      }
    },

    cachedWeek: (weekStart: string): WeekData | null => {
      const week = client.getQueryData<WeekData>(weekKey(weekStart));
      if (!week) return null;
      const projects = client.getQueryData<ResolvedProjects>(
        projectsKey(sortedProjectRefs(week)),
      );
      return projects ? applyResolvedProjects(week, projects) : week;
    },
  };
}

export function adjacentWeek(weekStart: string, offsetDays: number): string {
  const [y, m, d] = weekStart.split("-").map(Number);
  return toLocalISODate(new Date(y, m - 1, d + offsetDays));
}
