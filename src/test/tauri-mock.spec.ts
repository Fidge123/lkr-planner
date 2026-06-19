import { beforeEach, describe, expect, test } from "bun:test";
import { invoke, registerMock, reset } from "./tauri-mock";

describe("tauri-mock", () => {
  beforeEach(() => {
    reset();
  });

  test("throws a descriptive error for unregistered commands", async () => {
    await expect(invoke("load_local_store")).rejects.toThrow(
      'Unregistered Tauri command: "load_local_store"',
    );
  });

  test("returns the stub value for registered commands", async () => {
    registerMock("load_local_store", () => ({ employeeSettings: [] }));

    await expect(invoke("load_local_store")).resolves.toEqual({
      employeeSettings: [],
    });
  });

  test("passes invoke args through to the handler", async () => {
    registerMock("load_week_events", (args) => args?.weekStart);

    await expect(
      invoke("load_week_events", { weekStart: "2026-06-15" }),
    ).resolves.toBe("2026-06-15");
  });

  test("reset() clears all registered handlers", async () => {
    registerMock("load_local_store", () => ({ employeeSettings: [] }));
    reset();

    await expect(invoke("load_local_store")).rejects.toThrow(
      'Unregistered Tauri command: "load_local_store"',
    );
  });
});
