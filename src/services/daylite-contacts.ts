import {
  commands,
  type DayliteUpdateContactIcalUrlsInput,
  type PlanningContactRecord,
} from "../generated/tauri";
import { unwrapCommandResult } from "./command-result";
import { normalizeOptionalString } from "./daylite-service-helpers";
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

export async function updateDayliteContactIcalUrls(
  input: DayliteUpdateContactIcalUrlsInput,
): Promise<PlanningContactRecord> {
  const contact = unwrapCommandResult(
    await commands.dayliteUpdateContactIcalUrls(input),
    "Die Daten konnten nicht von Daylite geladen werden.",
  );

  contactCache.update((contacts) => {
    const others = contacts.filter((entry) => entry.self !== contact.self);
    return sortContacts(
      isMonteurContact(contact) ? [...others, contact] : others,
    );
  });

  return contact;
}

function isMonteurContact(contact: PlanningContactRecord): boolean {
  return normalizeOptionalString(contact.category)?.toLowerCase() === "monteur";
}

function sortContacts(
  contacts: PlanningContactRecord[],
): PlanningContactRecord[] {
  return [...contacts].sort((left, right) =>
    (left.nickname ?? left.full_name ?? "").localeCompare(
      right.nickname ?? right.full_name ?? "",
    ),
  );
}

export function test_resetDayliteContactCache(): void {
  contactCache.reset();
}
