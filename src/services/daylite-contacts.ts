import {
  type DayliteContactRecord,
  getDayliteContactDisplayName,
} from "../domain/planning";
import {
  commands,
  type DayliteUpdateContactIcalUrlsInput,
} from "../generated/tauri";
import {
  normalizeOptionalString,
  readDayliteApiErrorMessage,
} from "./daylite-service-helpers";

export const DEFAULT_DAYLITE_CONTACT_CACHE_TTL_MS = 30_000;

type DayliteContactsSource = "network" | "cache" | "disk-cache" | "stale-cache";

interface DayliteContactsLoadResult {
  contacts: DayliteContactRecord[];
  source: DayliteContactsSource;
  errorMessage?: string | null;
}

interface ContactCacheEntry {
  contacts: DayliteContactRecord[];
  fetchedAtMs: number;
}

interface DayliteContactsLoadOptions {
  nowMs?: number;
  forceRefresh?: boolean;
}

let cacheTtlMs = DEFAULT_DAYLITE_CONTACT_CACHE_TTL_MS;
let contactCache: ContactCacheEntry | null = null;
let inFlightRequest: Promise<DayliteContactsLoadResult> | null = null;

export async function loadDayliteContacts(
  options: DayliteContactsLoadOptions = {},
): Promise<DayliteContactsLoadResult> {
  const nowMs = options.nowMs ?? Date.now();
  const forceRefresh = options.forceRefresh ?? false;
  const cacheAgeMs = contactCache ? nowMs - contactCache.fetchedAtMs : Infinity;
  const cacheIsFresh = contactCache !== null && cacheAgeMs < cacheTtlMs;

  if (!forceRefresh && cacheIsFresh && contactCache) {
    return {
      contacts: contactCache.contacts,
      source: "cache",
    };
  }

  inFlightRequest ??= fetchContacts()
    .then((contacts) => {
      contactCache = { contacts, fetchedAtMs: nowMs };
      return {
        contacts,
        source: "network",
      } satisfies DayliteContactsLoadResult;
    })
    .catch(async (error) => {
      const errorMessage = getErrorMessage(error);

      if (contactCache) {
        return {
          contacts: contactCache.contacts,
          source: "stale-cache",
          errorMessage,
        } satisfies DayliteContactsLoadResult;
      }

      const contactsFromStore = await loadCachedDayliteContacts();
      if (contactsFromStore.length > 0) {
        contactCache = { contacts: contactsFromStore, fetchedAtMs: nowMs };
        return {
          contacts: contactsFromStore,
          source: "disk-cache",
          errorMessage,
        } satisfies DayliteContactsLoadResult;
      }

      throw new Error(`Mitarbeiterladen fehlgeschlagen: ${errorMessage}`);
    })
    .finally(() => {
      inFlightRequest = null;
    });

  return inFlightRequest;
}

export async function loadCachedDayliteContacts(): Promise<
  DayliteContactRecord[]
> {
  const result = await commands.dayliteListCachedContacts();
  if (result.status === "error") {
    return [];
  }

  return result.data;
}

export async function updateDayliteContactIcalUrls(
  input: DayliteUpdateContactIcalUrlsInput,
): Promise<DayliteContactRecord> {
  const result = await commands.dayliteUpdateContactIcalUrls(input);
  if (result.status === "error") {
    throw new Error(
      readDayliteApiErrorMessage(
        result.error,
        "Die Daten konnten nicht von Daylite geladen werden.",
      ),
    );
  }

  updateInMemoryContactCache(result.data);
  return result.data;
}

async function fetchContacts(): Promise<DayliteContactRecord[]> {
  const result = await commands.dayliteListContacts();
  if (result.status === "error") {
    throw new Error(
      readDayliteApiErrorMessage(
        result.error,
        "Die Daten konnten nicht von Daylite geladen werden.",
      ),
    );
  }

  return result.data;
}

function updateInMemoryContactCache(
  updatedContact: DayliteContactRecord,
): void {
  if (!contactCache) {
    return;
  }

  const contactsWithoutUpdated = contactCache.contacts.filter(
    (contact) => contact.self !== updatedContact.self,
  );

  if (isMonteurContact(updatedContact)) {
    contactsWithoutUpdated.push(updatedContact);
  }

  contactCache = {
    contacts: sortContacts(contactsWithoutUpdated),
    fetchedAtMs: contactCache.fetchedAtMs,
  };
}

function isMonteurContact(contact: DayliteContactRecord): boolean {
  return normalizeOptionalString(contact.category)?.toLowerCase() === "monteur";
}

function sortContacts(
  contacts: DayliteContactRecord[],
): DayliteContactRecord[] {
  return [...contacts].sort((left, right) =>
    getDayliteContactDisplayName(left).localeCompare(
      getDayliteContactDisplayName(right),
    ),
  );
}

function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  return "Die Daten konnten nicht von Daylite geladen werden.";
}

export function test_setDayliteContactCacheTtlMs(ttlMs: number): void {
  if (!Number.isFinite(ttlMs) || ttlMs <= 0) {
    cacheTtlMs = DEFAULT_DAYLITE_CONTACT_CACHE_TTL_MS;
    return;
  }

  cacheTtlMs = Math.floor(ttlMs);
}

export function test_resetDayliteContactCache(): void {
  contactCache = null;
  inFlightRequest = null;
}
