import type { DayliteProjectRecord } from "../domain/planning";
import { commands } from "../generated/tauri";

export const DEFAULT_DAYLITE_PROJECT_CACHE_TTL_MS = 30_000;

export type DayliteProjectsSource = "network" | "cache" | "stale-cache";

export interface DayliteProjectsLoadResult {
  projects: DayliteProjectRecord[];
  source: DayliteProjectsSource;
  errorMessage: string | null;
}

interface DayliteProjectCommandRecord {
  reference: string;
  name: string;
  status?: string;
  category?: string;
  keywords?: string[];
  due?: string | null;
  started?: string | null;
  completed?: string | null;
  createDate?: string | null;
  modifyDate?: string | null;
}

interface DayliteCommandError {
  userMessage?: string;
  user_message?: string;
}

interface DayliteCommandBindings {
  dayliteListProjects: () => Promise<
    | { status: "ok"; data: DayliteProjectCommandRecord[] }
    | { status: "error"; error: DayliteCommandError | string }
  >;
}

interface ProjectCacheEntry {
  projects: DayliteProjectRecord[];
  fetchedAtMs: number;
}

export interface DayliteProjectLoadOptions {
  nowMs?: number;
  forceRefresh?: boolean;
}

let cacheTtlMs = DEFAULT_DAYLITE_PROJECT_CACHE_TTL_MS;
let projectCache: ProjectCacheEntry | null = null;
let inFlightRequest: Promise<DayliteProjectsLoadResult> | null = null;

export async function loadDayliteProjects(
  options: DayliteProjectLoadOptions = {},
): Promise<DayliteProjectsLoadResult> {
  const nowMs = options.nowMs ?? Date.now();
  const forceRefresh = options.forceRefresh ?? false;
  const cacheAgeMs = projectCache ? nowMs - projectCache.fetchedAtMs : Infinity;
  const cacheIsFresh = projectCache !== null && cacheAgeMs < cacheTtlMs;

  if (!forceRefresh && cacheIsFresh && projectCache) {
    return {
      projects: projectCache.projects,
      source: "cache",
      errorMessage: null,
    };
  }

  if (inFlightRequest) {
    return inFlightRequest;
  }

  inFlightRequest = fetchAndMapProjects()
    .then((projects) => {
      projectCache = { projects, fetchedAtMs: nowMs };

      return {
        projects,
        source: "network",
        errorMessage: null,
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

export function setDayliteProjectCacheTtlMs(ttlMs: number): void {
  if (!Number.isFinite(ttlMs) || ttlMs <= 0) {
    cacheTtlMs = DEFAULT_DAYLITE_PROJECT_CACHE_TTL_MS;
    return;
  }

  cacheTtlMs = Math.floor(ttlMs);
}

export function resetDayliteProjectCacheForTests(): void {
  projectCache = null;
  inFlightRequest = null;
}

async function fetchAndMapProjects(): Promise<DayliteProjectRecord[]> {
  const dayliteCommands = commands as unknown as DayliteCommandBindings;
  if (typeof dayliteCommands.dayliteListProjects !== "function") {
    throw new Error(
      "Die Daylite-Projektfunktion ist nicht verfügbar. Bitte Anwendung neu starten.",
    );
  }

  const result = await dayliteCommands.dayliteListProjects();
  if (result.status === "error") {
    throw new Error(readCommandErrorMessage(result.error));
  }

  return result.data.map(mapDayliteProject);
}

function mapDayliteProject(
  project: DayliteProjectCommandRecord,
): DayliteProjectRecord {
  return {
    self: project.reference,
    name: project.name,
    status: normalizeStatus(project.status),
    category: normalizeOptionalString(project.category),
    keywords: normalizeStringArray(project.keywords),
    due: normalizeOptionalDate(project.due),
    started: normalizeOptionalDate(project.started),
    completed: normalizeOptionalDate(project.completed),
    create_date: normalizeOptionalDate(project.createDate),
    modify_date: normalizeOptionalDate(project.modifyDate),
  };
}

function normalizeStatus(
  status: string | undefined,
): DayliteProjectRecord["status"] {
  const normalized = status?.trim().toLowerCase();
  if (normalized === "new" || normalized === "new_status") {
    return "new_status";
  }
  if (normalized === "in_progress") {
    return "in_progress";
  }
  if (normalized === "done") {
    return "done";
  }
  if (normalized === "abandoned") {
    return "abandoned";
  }
  if (normalized === "cancelled") {
    return "cancelled";
  }
  if (normalized === "deferred") {
    return "deferred";
  }

  return "new_status";
}

function normalizeOptionalDate(
  value: string | null | undefined,
): string | undefined {
  if (!value || typeof value !== "string") {
    return undefined;
  }

  const parsedDate = new Date(value);
  if (Number.isNaN(parsedDate.getTime())) {
    return undefined;
  }

  return parsedDate.toISOString();
}

function normalizeOptionalString(
  value: string | undefined,
): string | undefined {
  if (typeof value !== "string") {
    return undefined;
  }

  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : undefined;
}

function normalizeStringArray(
  values: string[] | undefined,
): string[] | undefined {
  if (!Array.isArray(values)) {
    return undefined;
  }

  const normalized = values
    .filter((value): value is string => typeof value === "string")
    .map((value) => value.trim())
    .filter((value) => value.length > 0);

  return normalized.length > 0 ? normalized : undefined;
}

function readCommandErrorMessage(error: DayliteCommandError | string): string {
  if (typeof error === "string") {
    return error;
  }

  if (typeof error.userMessage === "string" && error.userMessage.length > 0) {
    return error.userMessage;
  }

  if (typeof error.user_message === "string" && error.user_message.length > 0) {
    return error.user_message;
  }

  return "Die Daten konnten nicht von Daylite geladen werden.";
}

function getErrorMessage(error: unknown): string {
  if (error instanceof Error && error.message.trim().length > 0) {
    return error.message;
  }

  return String(error);
}
