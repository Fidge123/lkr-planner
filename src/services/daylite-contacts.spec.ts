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
const mockDayliteUpdateContactIcalUrls = mock(() =>
  Promise.resolve({} as unknown),
);
const mockLoadLocalStore = mock(() =>
  Promise.resolve({
    status: "ok",
    data: {
      apiEndpoints: {
        dayliteBaseUrl: "https://daylite.example/v1",
        planradarBaseUrl: "",
      },
      tokenReferences: {
        dayliteTokenReference: "",
        planradarTokenReference: "",
      },
      employeeSettings: [],
      standardFilter: {
        pipelines: ["Aufträge"],
        columns: ["Vorbereitung", "Durchführung"],
        categories: ["Überfällig", "Liefertermin bekannt"],
        exclusionStatuses: ["Done"],
      },
      contactFilter: {
        activeEmployeeKeyword: "Monteur",
      },
      routingSettings: {
        openrouteserviceApiKey: "",
        openrouteserviceProfile: "driving-car",
      },
      dayliteCache: {
        lastSyncedAt: null,
        projects: [],
        contacts: [],
      },
    },
  } as const),
);
const mockSaveLocalStore = mock(() =>
  Promise.resolve({ status: "ok", data: null } as const),
);

mock.module("../generated/tauri", () => ({
  commands: {
    dayliteListContacts: mockDayliteListContacts,
    dayliteUpdateContactIcalUrls: mockDayliteUpdateContactIcalUrls,
    loadLocalStore: mockLoadLocalStore,
    saveLocalStore: mockSaveLocalStore,
  },
}));

describe("daylite contact service", () => {
  beforeEach(() => {
    mockDayliteListContacts.mockClear();
    mockDayliteUpdateContactIcalUrls.mockClear();
    mockLoadLocalStore.mockClear();
    mockSaveLocalStore.mockClear();
    resetDayliteContactCacheForTests();
    setDayliteContactCacheTtlMs(DEFAULT_DAYLITE_CONTACT_CACHE_TTL_MS);
  });

  it("maps contacts and keeps only category Monteur", async () => {
    mockDayliteListContacts.mockResolvedValue({
      status: "ok",
      data: [
        {
          self: "/v1/contacts/1001",
          fullName: "Max Mustermann",
          category: "Monteur",
          urls: [
            {
              label: "Einsatz iCal",
              url: "https://example.com/max-primary.ics",
            },
            {
              label: "Abwesenheit iCal",
              url: "https://example.com/max-absence.ics",
            },
          ],
        },
        {
          self: "/v1/contacts/1002",
          fullName: "Anna Vertrieb",
          category: "Vertrieb",
          urls: [],
        },
      ],
    });

    const result = await loadDayliteContacts({ nowMs: 1_000 });

    expect(result.source).toBe("network");
    expect(result.errorMessage).toBeNull();
    expect(result.contacts.map((contact) => contact.self)).toEqual([
      "/v1/contacts/1001",
    ]);
    expect(result.contacts[0]?.category).toBe("Monteur");
    expect(result.contacts[0]?.full_name).toBe("Max Mustermann");
  });

  it("persists mapped monteur contacts to local store cache", async () => {
    mockDayliteListContacts.mockResolvedValue({
      status: "ok",
      data: [
        {
          self: "/v1/contacts/2001",
          fullName: "Mona Monteur",
          category: "Monteur",
          urls: [
            {
              label: "Einsatz iCal",
              url: "https://example.com/mona-primary.ics",
            },
          ],
        },
      ],
    });

    await loadDayliteContacts({ nowMs: 2_000 });

    expect(mockSaveLocalStore).toHaveBeenCalledTimes(1);
    const savedStore = mockSaveLocalStore.mock.calls[0]?.[0];
    expect(savedStore.dayliteCache.contacts).toEqual([
      {
        reference: "/v1/contacts/2001",
        displayName: "Mona Monteur",
        fullName: "Mona Monteur",
        nickname: null,
        category: "Monteur",
        urls: [
          {
            label: "Einsatz iCal",
            url: "https://example.com/mona-primary.ics",
            note: null,
          },
        ],
      },
    ]);
  });

  it("loads cached monteur contacts from persisted local store", async () => {
    mockLoadLocalStore.mockResolvedValue({
      status: "ok",
      data: {
        apiEndpoints: {
          dayliteBaseUrl: "https://daylite.example/v1",
          planradarBaseUrl: "",
        },
        tokenReferences: {
          dayliteTokenReference: "",
          planradarTokenReference: "",
        },
        employeeSettings: [],
        standardFilter: {
          pipelines: ["Aufträge"],
          columns: ["Vorbereitung", "Durchführung"],
          categories: ["Überfällig", "Liefertermin bekannt"],
          exclusionStatuses: ["Done"],
        },
        contactFilter: {
          activeEmployeeKeyword: "Monteur",
        },
        routingSettings: {
          openrouteserviceApiKey: "",
          openrouteserviceProfile: "driving-car",
        },
        dayliteCache: {
          lastSyncedAt: "2026-02-20T08:00:00.000Z",
          projects: [],
          contacts: [
            {
              reference: "/v1/contacts/3001",
              displayName: "Moritz Monteur",
              fullName: "Moritz Monteur",
              nickname: null,
              category: "Monteur",
              urls: [
                {
                  label: "Abwesenheit iCal",
                  url: "https://example.com/moritz-absence.ics",
                  note: null,
                },
              ],
            },
            {
              reference: "/v1/contacts/3002",
              displayName: "Claudia Vertrieb",
              fullName: "Claudia Vertrieb",
              nickname: null,
              category: "Vertrieb",
              urls: [],
            },
          ],
        },
      },
    });

    const contacts = await loadCachedDayliteContactsFromStore();

    expect(contacts.map((contact) => contact.self)).toEqual([
      "/v1/contacts/3001",
    ]);
  });

  it("writes both iCal urls via daylite contact urls command", async () => {
    mockDayliteUpdateContactIcalUrls.mockResolvedValue({
      status: "ok",
      data: {
        self: "/v1/contacts/4001",
        fullName: "Mira Monteur",
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

    expect(updated.self).toBe("/v1/contacts/4001");
    expect(mockDayliteUpdateContactIcalUrls).toHaveBeenCalledWith({
      contactReference: "/v1/contacts/4001",
      primaryIcalUrl: "https://example.com/mira-primary.ics",
      absenceIcalUrl: "https://example.com/mira-absence.ics",
    });
  });
});
