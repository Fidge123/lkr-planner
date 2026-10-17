import { describe, expect, it } from "bun:test";
import type { EmployeeSetting } from "../generated/tauri";
import { calendarConfigChanged, runStartupSync } from "./startup-sync";

function setting(overrides: Partial<EmployeeSetting> = {}): EmployeeSetting {
  return {
    dayliteContactReference: "/v1/contacts/1",
    zepPrimaryCalendar: "https://zep/cal-1",
    zepAbsenceCalendar: null,
    ...overrides,
  };
}

describe("calendarConfigChanged", () => {
  it("sees no change when every calendar is the same", () => {
    expect(calendarConfigChanged([setting()], [setting()])).toBe(false);
  });

  it("ignores settings other than the calendars", () => {
    const tested = setting({ primaryIcalLastTestedAt: "2026-08-10T08:00:00Z" });

    expect(calendarConfigChanged([setting()], [tested])).toBe(false);
  });

  it("sees a changed primary calendar", () => {
    const after = setting({ zepPrimaryCalendar: "https://zep/cal-2" });

    expect(calendarConfigChanged([setting()], [after])).toBe(true);
  });

  it("sees a changed absence calendar", () => {
    const after = setting({ zepAbsenceCalendar: "https://zep/abs-1" });

    expect(calendarConfigChanged([setting()], [after])).toBe(true);
  });

  it("sees an employee who gained a calendar", () => {
    const added = setting({ dayliteContactReference: "/v1/contacts/2" });

    expect(calendarConfigChanged([setting()], [setting(), added])).toBe(true);
  });

  it("treats an employee without calendars like an absent one", () => {
    const empty = setting({
      dayliteContactReference: "/v1/contacts/2",
      zepPrimaryCalendar: null,
    });

    expect(calendarConfigChanged([setting()], [setting(), empty])).toBe(false);
  });

  it("treats an unreadable store like an empty one", () => {
    expect(calendarConfigChanged(null, null)).toBe(false);
    expect(calendarConfigChanged(null, [setting()])).toBe(true);
  });
});

function startup(
  reads: (EmployeeSetting[] | null)[],
  overrides: { syncContacts?: () => Promise<void> } = {},
) {
  const calls: string[] = [];
  let reloads = 0;
  const run = () =>
    runStartupSync({
      readSettings: async () => {
        calls.push("read");
        return reads.shift() ?? null;
      },
      syncContacts: async () => {
        calls.push("sync");
        await overrides.syncContacts?.();
      },
      reloadWeek: () => {
        reloads += 1;
      },
      isCancelled: () => false,
    });

  return { run, calls, reloads: () => reloads };
}

describe("runStartupSync", () => {
  it("reads the settings before syncing the contacts and again after", async () => {
    const { run, calls } = startup([[setting()], [setting()]]);

    await run();

    expect(calls).toEqual(["read", "sync", "read"]);
  });

  it("leaves the week alone when the sync changed no calendar", async () => {
    const { run, reloads } = startup([[setting()], [setting()]]);

    await run();

    expect(reloads()).toBe(0);
  });

  it("reloads the week when the sync changed a calendar", async () => {
    const { run, reloads } = startup([
      [setting()],
      [setting({ zepPrimaryCalendar: "https://zep/cal-2" })],
    ]);

    await run();

    expect(reloads()).toBe(1);
  });

  it("still compares the settings when the contact sync fails", async () => {
    const { run, calls, reloads } = startup([[setting()], [setting()]], {
      syncContacts: async () => {
        throw new Error("Daylite nicht erreichbar");
      },
    });

    await run();

    expect(calls).toEqual(["read", "sync", "read"]);
    expect(reloads()).toBe(0);
  });

  it("stops once cancelled", async () => {
    const calls: string[] = [];
    await runStartupSync({
      readSettings: async () => {
        calls.push("read");
        return [setting()];
      },
      syncContacts: async () => {
        calls.push("sync");
      },
      reloadWeek: () => calls.push("reload"),
      isCancelled: () => true,
    });

    expect(calls).toEqual(["read"]);
  });
});
