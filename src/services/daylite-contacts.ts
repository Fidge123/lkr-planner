import { commands, type PlanningContactRecord } from "../generated/tauri";
import { callCommand } from "./command-result";
import {
  type CacheLoadOptions,
  type CacheLoadResult,
  createTtlCache,
} from "./ttl-cache";

const DEFAULT_DAYLITE_CONTACT_CACHE_TTL_MS = 30_000;

const contactCache = createTtlCache<PlanningContactRecord>({
  ttlMs: DEFAULT_DAYLITE_CONTACT_CACHE_TTL_MS,
  failureMessage: "Mitarbeiterladen fehlgeschlagen",
  load: () =>
    callCommand(
      commands.dayliteListContacts(),
      "Die Daten konnten nicht von Daylite geladen werden.",
    ),
  fallback: () => loadCachedDayliteContacts(),
});

export function loadDayliteContacts(
  options: CacheLoadOptions = {},
): Promise<CacheLoadResult<PlanningContactRecord>> {
  return contactCache.get(options);
}

/**
 * Seeds the cache from disk, so the grid paints before the network answers.
 * A failing load is then served from memory instead of rereading the store.
 */
export async function preloadDayliteContactsFromDisk(): Promise<
  PlanningContactRecord[]
> {
  const contacts = await loadCachedDayliteContacts();
  contactCache.seed(contacts);
  return contacts;
}

export async function loadCachedDayliteContacts(): Promise<
  PlanningContactRecord[]
> {
  const result = await commands.dayliteListCachedContacts();
  if (result.status === "error") {
    return [];
  }

  return result.data;
}

export function test_resetDayliteContactCache(): void {
  contactCache.reset();
}
