import { describe, expect, it } from "bun:test";
import {
  reloadDataEvent,
  subscribeToReloadDataMenu,
} from "./use-reload-data-menu";

function fakeListen() {
  const state = { event: "", unlistened: false, fire: () => {} };
  const listenFn = async (event: string, handler: () => void) => {
    state.event = event;
    state.fire = handler;
    return () => {
      state.unlistened = true;
    };
  };
  return { state, listenFn };
}

describe("subscribeToReloadDataMenu", () => {
  it("reloads when the menu event fires", async () => {
    const { state, listenFn } = fakeListen();
    let reloads = 0;

    subscribeToReloadDataMenu(listenFn, () => {
      reloads += 1;
    });
    await Promise.resolve();
    state.fire();

    expect(state.event).toBe(reloadDataEvent);
    expect(reloads).toBe(1);
  });

  it("unlistens once the subscription resolves after cleanup", async () => {
    const { state, listenFn } = fakeListen();

    const cleanup = subscribeToReloadDataMenu(listenFn, () => {});
    cleanup();
    await Promise.resolve();
    await Promise.resolve();

    expect(state.unlistened).toBe(true);
  });
});
