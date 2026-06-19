import { describe, expect, test } from "bun:test";
import {
  createTauriMock,
  makeContacts,
  makeHolidays,
  makeLocalStore,
  makeWeekEvents,
} from "./tauri-mock-fixtures";

describe("tauri-mock fixtures", () => {
  test("makeLocalStore returns a valid LocalStore", () => {
    const store = makeLocalStore();

    expect(store.apiEndpoints.dayliteBaseUrl).toBeString();
    expect(store.employeeSettings).toEqual([]);
    expect(store.dayliteCache).toEqual({
      lastSyncedAt: null,
      projects: [],
      contacts: [],
    });
  });

  test("makeLocalStore applies overrides", () => {
    const store = makeLocalStore({
      employeeSettings: [],
      displaySettings: { hideNonPlannableEmployees: false },
    });

    expect(store.displaySettings).toEqual({ hideNonPlannableEmployees: false });
  });

  test("collection builders default to empty arrays", () => {
    expect(makeWeekEvents()).toEqual([]);
    expect(makeHolidays()).toEqual([]);
    expect(makeContacts()).toEqual([]);
  });
});

describe("createTauriMock", () => {
  test("collects registered responses keyed by command name", () => {
    const mock = createTauriMock();
    mock
      .registerMock("load_local_store", makeLocalStore())
      .registerMock("get_holidays_for_week", makeHolidays());

    expect(Object.keys(mock.responses)).toEqual([
      "load_local_store",
      "get_holidays_for_week",
    ]);
  });
});
