import type { DayliteProjectRecord } from "../domain/planning";
import { commands, type DayliteApiError } from "../generated/tauri";

export const DEFAULT_DAYLITE_PROJECT_CACHE_TTL_MS = 30_000;

type DayliteProjectsSource = "network" | "cache" | "stale-cache";

interface DayliteProjectsLoadResult {
  projects: DayliteProjectRecord[];
  source: DayliteProjectsSource;
  errorMessage?: string | null;
}

interface ProjectCacheEntry {
  projects: DayliteProjectRecord[];
  fetchedAtMs: number;
}

let cacheTtlMs = DEFAULT_DAYLITE_PROJECT_CACHE_TTL_MS;
let projectCache: ProjectCacheEntry | null = null;
let inFlightRequest: Promise<DayliteProjectsLoadResult> | null = null;

export async function loadDayliteProjects({
  nowMs = Date.now(),
  forceRefresh = false,
}): Promise<DayliteProjectsLoadResult> {
  const cacheAgeMs = projectCache ? nowMs - projectCache.fetchedAtMs : Infinity;
  const cacheIsFresh = projectCache !== null && cacheAgeMs < cacheTtlMs;

  if (!forceRefresh && cacheIsFresh && projectCache) {
    return {
      projects: projectCache.projects,
      source: "cache",
    };
  }
  inFlightRequest ??= fetchProjects()
    .then((projects) => {
      projectCache = { projects, fetchedAtMs: nowMs };
      return {
        projects,
        source: "network",
      } satisfies DayliteProjectsLoadResult;
    })
    .catch((error) => {
      const errorMessage = getErrorMessage(error);
      if (projectCache) {
        return {
          projects: projectCache.projects,
          source: "stale-cache",
          errorMessage,
        } satisfies DayliteProjectsLoadResult;
      }

      throw new Error(`Projektladen fehlgeschlagen: ${errorMessage}`);
    })
    .finally(() => {
      inFlightRequest = null;
    });

  return inFlightRequest;
}

async function fetchProjects(): Promise<DayliteProjectRecord[]> {
  const result = await commands.dayliteListProjects();
  if (result.status === "error") {
    throw new Error(readCommandErrorMessage(result.error));
  }

  return result.data;
}

function readCommandErrorMessage(error: DayliteApiError | string): string {
  if (typeof error === "string") {
    return error;
  }

  if (typeof error.userMessage === "string" && error.userMessage.length > 0) {
    return error.userMessage;
  }

  return "Die Daten konnten nicht von Daylite geladen werden.";
}

function getErrorMessage(error: unknown): string {
  if (error instanceof Error && error.message.trim().length > 0) {
    return error.message;
  }

  return String(error);
}

export function test_setDayliteProjectCacheTtlMs(ttlMs: number): void {
  if (!Number.isFinite(ttlMs) || ttlMs <= 0) {
    cacheTtlMs = DEFAULT_DAYLITE_PROJECT_CACHE_TTL_MS;
    return;
  }

  cacheTtlMs = Math.floor(ttlMs);
}

export function test_resetDayliteProjectCache(): void {
  projectCache = null;
  inFlightRequest = null;
}
