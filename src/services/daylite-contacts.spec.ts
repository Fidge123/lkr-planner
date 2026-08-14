import { beforeEach, describe, expect, it, mock } from "bun:test";
import {
  loadCachedDayliteContacts,
  loadDayliteContacts,
  preloadDayliteContactsFromDisk,
  test_resetDayliteContactCache,
} from "./daylite-contacts";

const mockDayliteListContacts = mock(() => Promise.resolve({} as unknown));
const mockDayliteListCachedContacts = mock(() =>
  Promise.resolve({} as unknown),
);

mock.module("../generated/tauri", () => ({
  commands: {
    dayliteListContacts: mockDayliteListContacts,
    dayliteListCachedContacts: mockDayliteListCachedContacts,
  },
}));

describe("daylite contact service", () => {
  beforeEach(() => {
    mockDayliteListContacts.mockClear();
    mockDayliteListCachedContacts.mockClear();
    test_resetDayliteContactCache();
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

    const result = await loadDayliteContacts();

    expect(result.source).toBe("network");
    expect(result.errorMessage).toBeUndefined();
    expect(result.data).toEqual([
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

    const result = await loadDayliteContacts();

    expect(result.source).toBe("disk-cache");
    expect(result.errorMessage).toBe(
      "Die Daten konnten nicht von Daylite geladen werden.",
    );
    expect(result.data[0]?.self).toBe("/v1/contacts/2001");
    expect(mockDayliteListCachedContacts).toHaveBeenCalledTimes(1);
  });

  it("reads the on-disk store once when the preload already served it", async () => {
    mockDayliteListCachedContacts.mockResolvedValue({
      status: "ok",
      data: [
        {
          self: "/v1/contacts/5001",
          full_name: "Malte Monteur",
          category: "Monteur",
          urls: [],
        },
      ],
    });
    mockDayliteListContacts.mockResolvedValue({
      status: "error",
      error: {
        userMessage: "Die Daten konnten nicht von Daylite geladen werden.",
      },
    });

    await preloadDayliteContactsFromDisk();
    const result = await loadDayliteContacts();

    expect(result.source).toBe("stale-cache");
    expect(result.data[0]?.self).toBe("/v1/contacts/5001");
    expect(mockDayliteListCachedContacts).toHaveBeenCalledTimes(1);
  });

  it("returns an empty list when cached contacts command fails", async () => {
    mockDayliteListCachedContacts.mockResolvedValue({
      status: "error",
      error: {
        userMessage: "Cache konnte nicht gelesen werden.",
      },
    });

    const contacts = await loadCachedDayliteContacts();

    expect(contacts).toEqual([]);
  });
});
