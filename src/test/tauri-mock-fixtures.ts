// Type-safe composition of Tauri mock responses for E2E tests.
//
// The mock replaces the raw `invoke`, which the generated bindings wrap in
// `typedError`, so a registered value is a command's success payload (the value
// `invoke` resolves to), not the wrapped `Result`. Both the registry and the
// fixture builders are typed against `src/generated/tauri.ts`, so a Rust type
// change plus regeneration surfaces as a single compile error here.

import type {
  commands,
  EmployeeWeekEvents,
  Holiday,
  LocalStore,
  PlanningContactRecord,
} from "../generated/tauri";

type Commands = typeof commands;

// Runtime dispatch keys on the snake_case command string the frontend passes to
// `invoke` (e.g. "load_local_store"), while the generated `commands` object keys
// on camelCase (e.g. loadLocalStore). This bridges the two so a registration
// keyed by the snake_case name resolves to the right command's payload type.
type CamelToSnake<S extends string> = S extends `${infer Head}${infer Tail}`
  ? `${Head extends Uppercase<Head> ? "_" : ""}${Lowercase<Head>}${CamelToSnake<Tail>}`
  : S;

type CommandKeyBySnake = {
  [K in keyof Commands & string as CamelToSnake<K>]: K;
};

export type CommandName = keyof CommandKeyBySnake;

type SuccessPayload<R> =
  Extract<Awaited<R>, { status: "ok" }> extends {
    data: infer D;
  }
    ? D
    : never;

export type MockPayload<N extends CommandName> = SuccessPayload<
  ReturnType<Commands[CommandKeyBySnake[N]]>
>;

export interface TauriMockComposer {
  responses: Record<string, unknown>;
  registerMock<N extends CommandName>(
    name: N,
    value: MockPayload<N>,
  ): TauriMockComposer;
}

export function createTauriMock(): TauriMockComposer {
  const responses: Record<string, unknown> = {};
  const composer: TauriMockComposer = {
    responses,
    registerMock(name, value) {
      responses[name] = value;
      return composer;
    },
  };
  return composer;
}

export function makeLocalStore(
  overrides: Partial<LocalStore> = {},
): LocalStore {
  return {
    apiEndpoints: {
      dayliteBaseUrl: "https://daylite.example",
      planradarBaseUrl: "https://planradar.example",
    },
    employeeSettings: [],
    dayliteCache: { lastSyncedAt: null, projects: [], contacts: [] },
    ...overrides,
  };
}

export function makeWeekEvents(
  events: EmployeeWeekEvents[] = [],
): EmployeeWeekEvents[] {
  return events;
}

export function makeHolidays(holidays: Holiday[] = []): Holiday[] {
  return holidays;
}

export function makeContacts(
  contacts: PlanningContactRecord[] = [],
): PlanningContactRecord[] {
  return contacts;
}
