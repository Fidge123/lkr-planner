import type { Page } from "@playwright/test";

// Installs the Tauri mock responses into the page before any application code
// runs. `addInitScript` executes on every navigation ahead of the bundle, so
// `invoke` calls made during initial render are covered. It also rebuilds the
// registry from scratch each time, so handler state never bleeds between tests
// that share the Vite server process.
export async function installTauriMock(
  page: Page,
  responses: Record<string, unknown>,
): Promise<void> {
  await page.addInitScript((data) => {
    const handlers: Record<
      string,
      (args?: Record<string, unknown>) => unknown
    > = {};
    for (const [command, value] of Object.entries(data)) {
      handlers[command] = () => value;
    }
    (window as unknown as Record<string, unknown>).__tauriMock = {
      handlers,
      registerMock(
        command: string,
        handler: (args?: Record<string, unknown>) => unknown,
      ) {
        handlers[command] = handler;
      },
      reset() {
        for (const key of Object.keys(handlers)) {
          delete handlers[key];
        }
      },
    };
  }, responses);
}
