import { GlobalRegistrator } from "@happy-dom/global-registrator";

GlobalRegistrator.register();

// Testing Library and the Tauri mocks must load after the DOM globals exist.
const { cleanup } = await import("@testing-library/react");
const { mockIPC } = await import("@tauri-apps/api/mocks");
const { afterEach } = await import("bun:test");

// Events reach `listen` through `emit`; a command no spec mocked fails loudly.
mockIPC(
  (command) => {
    throw new Error(`No mock for the "${command}" command`);
  },
  { shouldMockEvents: true },
);

afterEach(cleanup);
