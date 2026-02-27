import { beforeEach, describe, expect, it, mock } from "bun:test";
import {
  DEFAULT_DAYLITE_CONTACT_CACHE_TTL_MS,
  loadCachedDayliteContactsFromStore,
  loadDayliteContacts,
  resetDayliteContactCacheForTests,
  setDayliteContactCacheTtlMs,
  updateDayliteContactIcalUrls,
} from "./daylite-contacts";

const mockDayliteListContacts = mock(() => Promise.resolve({} as unknown));
const mockDayliteListCachedContacts = mock(() =>
  Promise.resolve({} as unknown),
);
const mockDayliteUpdateContactIcalUrls = mock(() =>
  Promise.resolve({} as unknown),
);

mock.module("../generated/tauri", () => ({
  commands: {
    dayliteListContacts: mockDayliteListContacts,
    dayliteListCachedContacts: mockDayliteListCachedContacts,
    dayliteUpdateContactIcalUrls: mockDayliteUpdateContactIcalUrls,
  },
}));

describe("daylite contact service", () => {
  beforeEach(() => {
    mockDayliteListContacts.mockClear();
    mockDayliteListCachedContacts.mockClear();
    mockDayliteUpdateContactIcalUrls.mockClear();
    resetDayliteContactCacheForTests();
    setDayliteContactCacheTtlMs(DEFAULT_DAYLITE_CONTACT_CACHE_TTL_MS);
  });

  it("returns planning contacts from backend command", async () => {
    mockDayliteListContacts.mockResolvedValue({
      status: "ok",
      data: [
        {
          self: "/v1/contacts/1001",
          full_name: "Max Mustermann",
          category: "Monteur",
          urls: [
            {
              label: "Einsatz iCal",
              url: "https://example.com/max-primary.ics",
            },
          ],
        },
      ],
    });

    const result = await loadDayliteContacts({ nowMs: 1_000 });

    expect(result.source).toBe("network");
    expect(result.errorMessage).toBeNull();
    expect(result.contacts).toEqual([
      {
        self: "/v1/contacts/1001",
        full_name: "Max Mustermann",
        nickname: undefined,
        category: "Monteur",
        urls: [
          {
            label: "Einsatz iCal",
            url: "https://example.com/max-primary.ics",
            note: undefined,
          },
        ],
      },
    ]);
  });

  it("falls back to cached contacts command when backend fails without memory cache", async () => {
    mockDayliteListContacts.mockResolvedValue({
      status: "error",
      error: {
        userMessage: "Die Daten konnten nicht von Daylite geladen werden.",
      },
    });
    mockDayliteListCachedContacts.mockResolvedValue({
      status: "ok",
      data: [
        {
          self: "/v1/contacts/2001",
          full_name: "Mona Monteur",
          category: "Monteur",
          urls: [],
        },
      ],
    });

    const result = await loadDayliteContacts({ nowMs: 2_000 });

    expect(result.source).toBe("disk-cache");
    expect(result.errorMessage).toBe(
      "Die Daten konnten nicht von Daylite geladen werden.",
    );
    expect(result.contacts[0]?.self).toBe("/v1/contacts/2001");
    expect(mockDayliteListCachedContacts).toHaveBeenCalledTimes(1);
  });

  it("returns stale in-memory cache on backend failure", async () => {
    mockDayliteListContacts
      .mockResolvedValueOnce({
        status: "ok",
        data: [
          {
            self: "/v1/contacts/3001",
            full_name: "Moritz Monteur",
            category: "Monteur",
            urls: [],
          },
        ],
      })
      .mockResolvedValueOnce({
        status: "error",
        error: {
          userMessage: "Die Daten konnten nicht von Daylite geladen werden.",
        },
      });

    await loadDayliteContacts({ nowMs: 1_000 });
    const staleFallback = await loadDayliteContacts({ nowMs: 45_000 });

    expect(staleFallback.source).toBe("stale-cache");
    expect(staleFallback.contacts[0]?.self).toBe("/v1/contacts/3001");
    expect(staleFallback.errorMessage).toBe(
      "Die Daten konnten nicht von Daylite geladen werden.",
    );
  });

  it("updates in-memory cache when contact urls are updated", async () => {
    mockDayliteListContacts.mockResolvedValue({
      status: "ok",
      data: [
        {
          self: "/v1/contacts/4001",
          full_name: "Mira Monteur",
          category: "Monteur",
          urls: [],
        },
      ],
    });
    await loadDayliteContacts({ nowMs: 1_000 });

    mockDayliteUpdateContactIcalUrls.mockResolvedValue({
      status: "ok",
      data: {
        self: "/v1/contacts/4001",
        full_name: "Mira Monteur (Aktualisiert)",
        category: "Monteur",
        urls: [
          {
            label: "Einsatz iCal",
            url: "https://example.com/mira-primary.ics",
          },
          {
            label: "Abwesenheit iCal",
            url: "https://example.com/mira-absence.ics",
          },
        ],
      },
    });

    const updated = await updateDayliteContactIcalUrls({
      contactReference: "/v1/contacts/4001",
      primaryIcalUrl: "https://example.com/mira-primary.ics",
      absenceIcalUrl: "https://example.com/mira-absence.ics",
    });

    expect(updated.full_name).toBe("Mira Monteur (Aktualisiert)");

    const cached = await loadDayliteContacts({ nowMs: 1_001 });
    expect(cached.source).toBe("cache");
    expect(cached.contacts[0]?.full_name).toBe("Mira Monteur (Aktualisiert)");
  });

  it("returns an empty list when cached contacts command fails", async () => {
    mockDayliteListCachedContacts.mockResolvedValue({
      status: "error",
      error: {
        userMessage: "Cache konnte nicht gelesen werden.",
      },
    });

    const contacts = await loadCachedDayliteContactsFromStore();

    expect(contacts).toEqual([]);
  });
});
