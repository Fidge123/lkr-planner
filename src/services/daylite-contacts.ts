import {
  type DayliteContactRecord,
  getDayliteContactDisplayName,
} from "../domain/planning";
import { commands } from "../generated/tauri";

export const DEFAULT_DAYLITE_CONTACT_CACHE_TTL_MS = 30_000;

export type DayliteContactsSource =
  | "network"
  | "cache"
  | "disk-cache"
  | "stale-cache";

export interface DayliteContactsLoadResult {
  contacts: DayliteContactRecord[];
  source: DayliteContactsSource;
  errorMessage: string | null;
}

export interface DayliteContactIcalUpdateInput {
  contactReference: string;
  primaryIcalUrl: string;
  absenceIcalUrl: string;
}

interface DayliteContactCommandUrlRecord {
  label?: string | null;
  url?: string | null;
  note?: string | null;
}

interface DayliteContactCommandRecord {
  reference?: string;
  self?: string;
  firstName?: string;
  lastName?: string;
  fullName?: string;
  nickname?: string;
  category?: string;
  urls?: DayliteContactCommandUrlRecord[];
}

interface DayliteCommandError {
  userMessage?: string;
  user_message?: string;
}

interface LocalStoreContactCacheEntry {
  reference: string;
  displayName?: string;
  fullName?: string | null;
  nickname?: string | null;
  category?: string | null;
  urls?: DayliteContactCommandUrlRecord[];
}

interface LocalStoreData {
  dayliteCache?: {
    lastSyncedAt?: string | null;
    contacts?: LocalStoreContactCacheEntry[];
  };
}

interface DayliteCommandBindings {
  dayliteListContacts: () => Promise<
    | { status: "ok"; data: DayliteContactCommandRecord[] }
    | { status: "error"; error: DayliteCommandError | string }
  >;
  dayliteUpdateContactIcalUrls: (
    input: DayliteContactIcalUpdateInput,
  ) => Promise<
    | { status: "ok"; data: DayliteContactCommandRecord }
    | { status: "error"; error: DayliteCommandError | string }
  >;
  loadLocalStore: () => Promise<
    | { status: "ok"; data: LocalStoreData }
    | { status: "error"; error: { userMessage: string } | string }
  >;
  saveLocalStore: (
    store: LocalStoreData,
  ) => Promise<
    | { status: "ok"; data: null }
    | { status: "error"; error: { userMessage: string } | string }
  >;
}

interface ContactCacheEntry {
  contacts: DayliteContactRecord[];
  fetchedAtMs: number;
}

export interface DayliteContactsLoadOptions {
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
      errorMessage: null,
    };
  }

  if (inFlightRequest) {
    return inFlightRequest;
  }

  inFlightRequest = fetchAndMapContacts()
    .then(async (contacts) => {
      contactCache = { contacts, fetchedAtMs: nowMs };
      await persistContactsToStore(contacts, nowMs);

      return {
        contacts,
        source: "network",
        errorMessage: null,
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

      const contactsFromStore = await loadCachedDayliteContactsFromStore();
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

export async function loadCachedDayliteContactsFromStore(): Promise<
  DayliteContactRecord[]
> {
  const dayliteCommands = commands as unknown as DayliteCommandBindings;
  if (typeof dayliteCommands.loadLocalStore !== "function") {
    return [];
  }

  const result = await dayliteCommands.loadLocalStore();
  if (result.status === "error") {
    return [];
  }

  const cachedContacts = result.data.dayliteCache?.contacts ?? [];
  const mappedContacts = cachedContacts.map(mapCachedContact);
  const filteredContacts = filterMonteurContacts(mappedContacts);
  return sortContacts(filteredContacts);
}

export async function updateDayliteContactIcalUrls(
  input: DayliteContactIcalUpdateInput,
): Promise<DayliteContactRecord> {
  const dayliteCommands = commands as unknown as DayliteCommandBindings;
  if (typeof dayliteCommands.dayliteUpdateContactIcalUrls !== "function") {
    throw new Error(
      "Die Daylite-Kontaktfunktion ist nicht verfügbar. Bitte Anwendung neu starten.",
    );
  }

  const result = await dayliteCommands.dayliteUpdateContactIcalUrls(input);
  if (result.status === "error") {
    throw new Error(readCommandErrorMessage(result.error));
  }

  const updatedContact = mapDayliteContact(result.data);

  await persistUpdatedContactToStore(updatedContact);
  updateInMemoryContactCache(updatedContact);

  return updatedContact;
}

export function setDayliteContactCacheTtlMs(ttlMs: number): void {
  if (!Number.isFinite(ttlMs) || ttlMs <= 0) {
    cacheTtlMs = DEFAULT_DAYLITE_CONTACT_CACHE_TTL_MS;
    return;
  }

  cacheTtlMs = Math.floor(ttlMs);
}

export function resetDayliteContactCacheForTests(): void {
  contactCache = null;
  inFlightRequest = null;
}

async function fetchAndMapContacts(): Promise<DayliteContactRecord[]> {
  const dayliteCommands = commands as unknown as DayliteCommandBindings;
  if (typeof dayliteCommands.dayliteListContacts !== "function") {
    throw new Error(
      "Die Daylite-Kontaktfunktion ist nicht verfügbar. Bitte Anwendung neu starten.",
    );
  }

  const result = await dayliteCommands.dayliteListContacts();
  if (result.status === "error") {
    throw new Error(readCommandErrorMessage(result.error));
  }

  const contacts = result.data.map(mapDayliteContact);
  const filteredContacts = filterMonteurContacts(contacts);
  return sortContacts(filteredContacts);
}

async function persistContactsToStore(
  contacts: DayliteContactRecord[],
  nowMs: number,
): Promise<void> {
  const dayliteCommands = commands as unknown as DayliteCommandBindings;
  if (
    typeof dayliteCommands.loadLocalStore !== "function" ||
    typeof dayliteCommands.saveLocalStore !== "function"
  ) {
    return;
  }

  const loadResult = await dayliteCommands.loadLocalStore();
  if (loadResult.status === "error") {
    return;
  }

  const updatedStore: LocalStoreData = {
    ...loadResult.data,
    dayliteCache: {
      ...loadResult.data.dayliteCache,
      lastSyncedAt: new Date(nowMs).toISOString(),
      contacts: contacts.map(mapContactToCacheEntry),
    },
  };

  await dayliteCommands.saveLocalStore(updatedStore);
}

async function persistUpdatedContactToStore(
  updatedContact: DayliteContactRecord,
): Promise<void> {
  const dayliteCommands = commands as unknown as DayliteCommandBindings;
  if (
    typeof dayliteCommands.loadLocalStore !== "function" ||
    typeof dayliteCommands.saveLocalStore !== "function"
  ) {
    return;
  }

  const loadResult = await dayliteCommands.loadLocalStore();
  if (loadResult.status === "error") {
    return;
  }

  const cachedContacts =
    loadResult.data.dayliteCache?.contacts?.map(mapCachedContact) ?? [];
  const filteredExistingContacts = cachedContacts.filter(
    (contact) => contact.self !== updatedContact.self,
  );

  if (isMonteurContact(updatedContact)) {
    filteredExistingContacts.push(updatedContact);
  }

  const sortedContacts = sortContacts(
    filterMonteurContacts(filteredExistingContacts),
  );

  const updatedStore: LocalStoreData = {
    ...loadResult.data,
    dayliteCache: {
      ...loadResult.data.dayliteCache,
      lastSyncedAt: new Date().toISOString(),
      contacts: sortedContacts.map(mapContactToCacheEntry),
    },
  };

  await dayliteCommands.saveLocalStore(updatedStore);
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
    contacts: sortContacts(filterMonteurContacts(contactsWithoutUpdated)),
    fetchedAtMs: contactCache.fetchedAtMs,
  };
}

function mapDayliteContact(
  contact: DayliteContactCommandRecord,
): DayliteContactRecord {
  const firstName = normalizeOptionalString(contact.firstName);
  const lastName = normalizeOptionalString(contact.lastName);
  const reference =
    normalizeOptionalString(contact.reference) ??
    normalizeOptionalString(contact.self) ??
    "";
  const fullName =
    normalizeOptionalString(contact.fullName) ?? joinName(firstName, lastName);

  return {
    self: reference,
    full_name: fullName,
    nickname: normalizeOptionalString(contact.nickname),
    category: normalizeOptionalString(contact.category),
    urls: normalizeUrlList(contact.urls),
  };
}

function mapCachedContact(
  contact: LocalStoreContactCacheEntry,
): DayliteContactRecord {
  return {
    self: contact.reference,
    full_name:
      normalizeOptionalString(contact.fullName ?? undefined) ??
      normalizeOptionalString(contact.displayName),
    nickname: normalizeOptionalString(contact.nickname ?? undefined),
    category: normalizeOptionalString(contact.category ?? undefined),
    urls: normalizeUrlList(contact.urls),
  };
}

function mapContactToCacheEntry(
  contact: DayliteContactRecord,
): LocalStoreContactCacheEntry {
  return {
    reference: contact.self,
    displayName: getDayliteContactDisplayName(contact),
    fullName: contact.full_name ?? null,
    nickname: contact.nickname ?? null,
    category: contact.category ?? null,
    urls:
      contact.urls?.map((url) => ({
        label: url.label ?? null,
        url: url.url ?? null,
        note: url.note ?? null,
      })) ?? [],
  };
}

function filterMonteurContacts(
  contacts: DayliteContactRecord[],
): DayliteContactRecord[] {
  return contacts.filter(isMonteurContact);
}

function isMonteurContact(contact: DayliteContactRecord): boolean {
  return normalizeOptionalString(contact.category)?.toLowerCase() === "monteur";
}

function sortContacts(
  contacts: DayliteContactRecord[],
): DayliteContactRecord[] {
  return [...contacts].sort((leftContact, rightContact) =>
    getDayliteContactDisplayName(leftContact).localeCompare(
      getDayliteContactDisplayName(rightContact),
      "de-DE",
      { sensitivity: "base" },
    ),
  );
}

function normalizeOptionalString(
  value: string | null | undefined,
): string | undefined {
  if (typeof value !== "string") {
    return undefined;
  }

  const normalized = value.trim();
  return normalized.length > 0 ? normalized : undefined;
}

function normalizeUrlList(
  urls: DayliteContactCommandUrlRecord[] | undefined,
): { label?: string; url?: string; note?: string }[] | undefined {
  if (!Array.isArray(urls)) {
    return undefined;
  }

  const normalizedUrls = urls
    .map((url) => ({
      label: normalizeOptionalString(url.label ?? undefined),
      url: normalizeOptionalString(url.url ?? undefined),
      note: normalizeOptionalString(url.note ?? undefined),
    }))
    .filter((url) => url.label || url.url || url.note);

  return normalizedUrls.length > 0 ? normalizedUrls : undefined;
}

function joinName(
  firstName: string | undefined,
  lastName: string | undefined,
): string | undefined {
  const fullName = [firstName, lastName].filter(Boolean).join(" ").trim();
  return fullName.length > 0 ? fullName : undefined;
}

function readCommandErrorMessage(error: DayliteCommandError | string): string {
  if (typeof error === "string") {
    return error;
  }

  return (
    error.userMessage ??
    error.user_message ??
    "Die Daten konnten nicht von Daylite geladen werden."
  );
}

function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  return "Die Daten konnten nicht von Daylite geladen werden.";
}
