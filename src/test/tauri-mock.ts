// Browser-side replacement for `@tauri-apps/api/core` used in Playwright E2E
// tests. A Vite alias (see `vite.playwright.config.ts`) swaps the real module
// for this one, so the generated bindings in `src/generated/tauri.ts` invoke
// these stubs instead of the Tauri backend.
//
// The handler registry lives on a global so a Playwright `addInitScript` can
// populate it before any application code runs (see
// `tests/e2e/support/tauri-mock-page.ts`). The same module is exercised
// directly by a Bun unit test, where no init script runs and the registry is
// created lazily on first use.

export type TauriMockHandler = (args?: Record<string, unknown>) => unknown;

export interface TauriMockRegistry {
  handlers: Record<string, TauriMockHandler>;
  registerMock(command: string, handler: TauriMockHandler): void;
  reset(): void;
}

const GLOBAL_KEY = "__tauriMock";

function createRegistry(): TauriMockRegistry {
  const handlers: Record<string, TauriMockHandler> = {};
  return {
    handlers,
    registerMock(command, handler) {
      handlers[command] = handler;
    },
    reset() {
      for (const key of Object.keys(handlers)) {
        delete handlers[key];
      }
    },
  };
}

// Read the registry lazily on every call so an init script that installs
// `window.__tauriMock` after this module loads is still picked up.
function getRegistry(): TauriMockRegistry {
  const scope = globalThis as Record<string, unknown>;
  if (!scope[GLOBAL_KEY]) {
    scope[GLOBAL_KEY] = createRegistry();
  }
  return scope[GLOBAL_KEY] as TauriMockRegistry;
}

export function registerMock(command: string, handler: TauriMockHandler): void {
  getRegistry().registerMock(command, handler);
}

export function reset(): void {
  getRegistry().reset();
}

// Matches the `invoke<T>(cmd, args?)` signature of `@tauri-apps/api/core`.
export async function invoke<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const handler = getRegistry().handlers[command];
  if (!handler) {
    throw new Error(`Unregistered Tauri command: "${command}"`);
  }
  return handler(args) as T;
}
