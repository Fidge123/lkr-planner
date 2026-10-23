import { describe, expect, it } from "bun:test";
import { subscribeToTauriEvent } from "./tauri-event";

function fakeListen() {
  const state = {
    event: "",
    unlistened: false,
    fire: (_payload: string[]) => {},
  };
  const listenFn = async (
    event: string,
    handler: (event: { payload: string[] }) => void,
  ) => {
    state.event = event;
    state.fire = (payload) => handler({ payload });
    return () => {
      state.unlistened = true;
    };
  };
  return { state, listenFn };
}

describe("subscribeToTauriEvent", () => {
  it("passes each payload of the named event on", async () => {
    const { state, listenFn } = fakeListen();
    const received: string[][] = [];

    subscribeToTauriEvent(listenFn, "reload-data", (payload) => {
      received.push(payload);
    });
    await Promise.resolve();
    state.fire(["2026-11-02"]);
    state.fire([]);

    expect(state.event).toBe("reload-data");
    expect(received).toEqual([["2026-11-02"], []]);
  });

  it("unlistens once the subscription resolves after cleanup", async () => {
    const { state, listenFn } = fakeListen();

    const cleanup = subscribeToTauriEvent(listenFn, "reload-data", () => {});
    cleanup();
    await Promise.resolve();
    await Promise.resolve();

    expect(state.unlistened).toBe(true);
  });
});
