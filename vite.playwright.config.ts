// Vite config used only when running Playwright E2E tests. It extends the base
// config and swaps `@tauri-apps/api/core` for the browser mock so `invoke`
// calls resolve to test stubs. A dedicated port keeps it clear of `bun dev`
// (which uses 1420 with strictPort).

import { fileURLToPath } from "node:url";
import { defineConfig, mergeConfig, type UserConfigFnObject } from "vite";
import baseConfig from "./vite.config";

export default defineConfig(async (env) => {
  const base = await (baseConfig as UserConfigFnObject)(env);
  return mergeConfig(base, {
    resolve: {
      alias: {
        "@tauri-apps/api/core": fileURLToPath(
          new URL("./src/test/tauri-mock.ts", import.meta.url),
        ),
      },
    },
    server: {
      port: 5174,
      strictPort: true,
    },
  });
});
