import { expect, test } from "@playwright/test";

// Minimal smoke of the test harness itself: the Vite dev server starts and the
// document loads. It does not register any mocks because it only asserts the
// HTML document, not application behavior.
test("app document loads", async ({ page }) => {
  const response = await page.goto("/");

  expect(response?.ok()).toBe(true);
  await expect(page).toHaveTitle(/LKR Planner/);
});
