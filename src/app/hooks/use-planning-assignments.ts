import {
  keepPreviousData,
  type QueryClient,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { useEffect, useMemo } from "react";
import type { CalendarCellEvent } from "../../generated/tauri";
import {
  loadWeekEvents,
  resolveWeekProjects,
} from "../../services/assignments";
import {
  adjacentWeek,
  createWeekCache,
  joinProjects,
  lastLoadedAt,
  sortedProjectRefs,
  type WeekCache,
} from "../week-cache";
import type { WeekData } from "../week-data";
import { useLeadingDebounce } from "./use-leading-debounce";

type EmployeeEvents = Record<string, CalendarCellEvent[]>;
type EmployeeErrors = Record<string, string>;

export type { WeekData };

export interface PlanningAssignmentsView {
  eventsByEmployee: EmployeeEvents;
  errorsByEmployee: EmployeeErrors;
  isLoading: boolean;
  errorMessage: string | null;
  /** The week the shown events belong to, or null while it loads. */
  loadedWeekStart: string | null;
  /** When the shown week's data was fetched, or when its load failed. */
  lastLoadedAt: number | null;
}

export interface PlanningAssignmentsState extends PlanningAssignmentsView {
  /** Loads the displayed week and its projects again, bypassing both caches. */
  reloadAssignments: () => void;
  /** For automatic refreshes: keeps the shown data on failure and retries once before showing an error. */
  refreshAssignments: () => void;
  getCachedWeek: (weekStart: string) => WeekData | null;
}

const prefetchIdleMs = 400;
const emptyWeek: WeekData = { eventsByEmployee: {}, errorsByEmployee: {} };
const caches = new WeakMap<QueryClient, WeekCache>();

/** Every caller shares one cache per query client, so edits and refreshes see each other. */
export function useWeekCache(): WeekCache {
  const client = useQueryClient();
  let cache = caches.get(client);
  if (!cache) {
    cache = createWeekCache(client, { loadWeekEvents, resolveWeekProjects });
    caches.set(client, cache);
  }
  return cache;
}

/** A null week start holds the first load back until the week is known. */
export function usePlanningAssignments(
  weekStart: string | null,
): PlanningAssignmentsState {
  // The debounce absorbs rapid navigation; the first known week must not wait it out.
  const shownWeek = useLeadingDebounce(weekStart, 200) ?? weekStart;
  const cache = useWeekCache();
  const week = useQuery({
    ...cache.weekQuery(shownWeek ?? ""),
    enabled: shownWeek !== null,
  });
  const refs = useMemo(
    () => (week.data ? sortedProjectRefs(week.data) : []),
    [week.data],
  );
  const projects = useQuery({
    ...cache.projectsQuery(refs),
    placeholderData: keepPreviousData,
  });

  // A started week load cannot be called off, so prefetching waits for the user to settle first.
  useEffect(() => {
    if (shownWeek === null || !week.isSuccess) return;
    const timer = setTimeout(() => {
      void cache.prefetch(adjacentWeek(shownWeek, -7));
      void cache.prefetch(adjacentWeek(shownWeek, 7));
    }, prefetchIdleMs);
    return () => clearTimeout(timer);
  }, [cache, shownWeek, week.isSuccess]);

  const shown = useMemo(
    () =>
      week.data
        ? joinProjects(week.data, refs, projects.data, projects.isError)
        : emptyWeek,
    [week.data, refs, projects.data, projects.isError],
  );

  const actions = useMemo(
    () => ({
      reloadAssignments: () => {
        if (weekStart !== null) void cache.reload(weekStart);
      },
      refreshAssignments: () => {
        if (shownWeek !== null) void cache.refresh(shownWeek);
      },
      getCachedWeek: cache.cachedWeek,
    }),
    [cache, weekStart, shownWeek],
  );

  return {
    ...actions,
    eventsByEmployee: shown.eventsByEmployee,
    errorsByEmployee: shown.errorsByEmployee,
    // A failed automatic refresh waits for its retry without the spinner.
    isLoading: week.isFetching && week.failureCount === 0,
    errorMessage: week.error?.message ?? null,
    loadedWeekStart: week.data ? shownWeek : null,
    lastLoadedAt: lastLoadedAt(week),
  };
}
