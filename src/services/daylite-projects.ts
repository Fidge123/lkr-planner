import { commands, type PlanningProjectRecord } from "../generated/tauri";
import { unwrapCommandResult } from "./command-result";
import {
  type CacheLoadOptions,
  type CacheLoadResult,
  createTtlCache,
} from "./ttl-cache";

const DEFAULT_DAYLITE_PROJECT_CACHE_TTL_MS = 30_000;

const projectCache = createTtlCache<PlanningProjectRecord>({
  ttlMs: DEFAULT_DAYLITE_PROJECT_CACHE_TTL_MS,
  failureMessage: "Projektladen fehlgeschlagen",
  load: async () =>
    unwrapCommandResult(
      await commands.dayliteListProjects(),
      "Die Daten konnten nicht von Daylite geladen werden.",
    ),
});

export function loadDayliteProjects(
  options: CacheLoadOptions = {},
): Promise<CacheLoadResult<PlanningProjectRecord>> {
  return projectCache.get(options);
}

export function test_resetDayliteProjectCache(): void {
  projectCache.reset();
}
