import { commands, type PlanningContactRecord } from "../generated/tauri";
import { unwrapCommandResult } from "./command-result";
import {
  type CacheLoadOptions,
  type CacheLoadResult,
  createTtlCache,
} from "./ttl-cache";

const DEFAULT_DAYLITE_CONTACT_CACHE_TTL_MS = 30_000;

const contactCache = createTtlCache<PlanningContactRecord>({
  ttlMs: DEFAULT_DAYLITE_CONTACT_CACHE_TTL_MS,
  failureMessage: "Mitarbeiterladen fehlgeschlagen",
  load: async () =>
    unwrapCommandResult(
      await commands.dayliteListContacts(),
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
 * Publishes the on-disk contacts into the cache so the grid can paint before the
 * network answers, and so a failing load is served from memory instead of reading
 * the same store a second time.
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
