import { expect, test } from "@playwright/test";
import {
  createTauriMock,
  makeContacts,
  makeHolidays,
  makeLocalStore,
  makeWeekEvents,
} from "../../src/test/tauri-mock-fixtures";
import { installTauriMock } from "./support/tauri-mock-page";

test("planning view renders without JavaScript errors", async ({ page }) => {
  const pageErrors: string[] = [];
  page.on("pageerror", (error) => pageErrors.push(error.message));
  // Unregistered commands can surface as unhandled promise rejections rather
  // than uncaught exceptions, which `pageerror` does not report, so capture
  // those too for an honest "no JavaScript errors" assertion.
  await page.addInitScript(() => {
    const scope = window as unknown as { __rejections?: string[] };
    scope.__rejections = [];
    window.addEventListener("unhandledrejection", (event) => {
      scope.__rejections?.push(String(event.reason));
    });
  });

  // The mount effects fire these commands during initial render. The mock
  // throws for anything unregistered, so every command the startup path can
  // reach must be stubbed. daylite_list_contacts (Daylite sync) and
  // daylite_list_cached_contacts (planning employees hook) both run on mount,
  // alongside the planning data (load_local_store, load_week_events,
  // get_holidays_for_week) and the settings dialog (zep_load_credentials).
  const mock = createTauriMock();
  mock
    .registerMock("daylite_list_contacts", makeContacts())
    .registerMock("daylite_list_cached_contacts", makeContacts())
    .registerMock("load_local_store", makeLocalStore())
    .registerMock("load_week_events", makeWeekEvents())
    .registerMock("get_holidays_for_week", makeHolidays())
    .registerMock("zep_load_credentials", null);

  await installTauriMock(page, mock.responses);
  await page.goto("/");

  await expect(page.getByTestId("planning-view")).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Wochenplanung" }),
  ).toBeVisible();

  const rejections = await page.evaluate(
    () => (window as unknown as { __rejections?: string[] }).__rejections ?? [],
  );
  expect(pageErrors).toEqual([]);
  expect(rejections).toEqual([]);
});
