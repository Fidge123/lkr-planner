// Test preload: registers a DOM (happy-dom) so component tests can render React
// trees under `bun test`, and clears the DOM after each test. Wired in via
// `bunfig.toml`'s `[test].preload`.
import { GlobalRegistrator } from "@happy-dom/global-registrator";

GlobalRegistrator.register();

const { cleanup } = await import("@testing-library/react");
const { afterEach } = await import("bun:test");

afterEach(() => {
  cleanup();
});
