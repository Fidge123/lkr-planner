import { defineConfig, devices } from "@playwright/test";

// E2E tests use the `.e2e.ts` suffix (not `.spec.ts`) so the native `bun test`
// runner, which scans the whole repo for `*.spec.ts`, does not try to execute
// Playwright tests.
export default defineConfig({
  testDir: "./tests/e2e",
  testMatch: "**/*.e2e.ts",
  fullyParallel: true,
  use: {
    baseURL: "http://localhost:5174",
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
  webServer: {
    command: "bunx vite --config vite.playwright.config.ts",
    port: 5174,
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});
