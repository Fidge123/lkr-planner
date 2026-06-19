// Type-level test for the mock registry. It is checked by `tsc --noEmit`
// (`bun run typecheck`, and gating `bun test:e2e`), not by the Bun runner, so
// the filename intentionally avoids the `.spec.ts` / `.test.ts` pattern.
//
// Each `@ts-expect-error` asserts a compile failure: if `registerMock` ever
// stops constraining its value, the bad call no longer errors, the directive
// becomes unused, and `tsc` fails. That is the RED state before `registerMock`
// is typed against the generated bindings.

import { createTauriMock, makeLocalStore } from "./tauri-mock-fixtures";

const mock = createTauriMock();

// A value matching the generated `LocalStore` payload is accepted.
mock.registerMock("load_local_store", makeLocalStore());

// A value not assignable to `LocalStore` is rejected.
// @ts-expect-error - missing required LocalStore fields
mock.registerMock("load_local_store", { not: "a local store" });

// An unknown command name is rejected.
// @ts-expect-error - not a generated command name
mock.registerMock("not_a_real_command", makeLocalStore());

// A holiday list is accepted for the holidays command but not for the store.
mock.registerMock("get_holidays_for_week", []);
// @ts-expect-error - LocalStore is not assignable to Holiday[]
mock.registerMock("get_holidays_for_week", makeLocalStore());
