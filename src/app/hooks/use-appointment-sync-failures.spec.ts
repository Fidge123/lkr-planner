import { describe, expect, it } from "bun:test";
import {
  appointmentSyncFailuresEvent,
  createRetryRunner,
  subscribeToAppointmentSyncFailures,
} from "./use-appointment-sync-failures";

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

function deferred() {
  let resolve = () => {};
  let reject = (_error: Error) => {};
  const promise = new Promise<void>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

describe("subscribeToAppointmentSyncFailures", () => {
  it("listens to the failures event", async () => {
    const { state, listenFn } = fakeListen();

    subscribeToAppointmentSyncFailures(listenFn, () => {});
    await Promise.resolve();

    expect(state.event).toBe(appointmentSyncFailuresEvent);
    expect(appointmentSyncFailuresEvent).toBe(
      "daylite-appointment-sync-failures",
    );
  });

  it("replaces the failed dates with each event payload", async () => {
    const { state, listenFn } = fakeListen();
    const received: string[][] = [];

    subscribeToAppointmentSyncFailures(listenFn, (dates) => {
      received.push(dates);
    });
    await Promise.resolve();
    state.fire(["2026-11-02", "2026-11-03"]);
    state.fire([]);

    expect(received).toEqual([["2026-11-02", "2026-11-03"], []]);
  });

  it("unlistens once the subscription resolves after cleanup", async () => {
    const { state, listenFn } = fakeListen();

    const cleanup = subscribeToAppointmentSyncFailures(listenFn, () => {});
    cleanup();
    await Promise.resolve();
    await Promise.resolve();

    expect(state.unlistened).toBe(true);
  });
});

describe("createRetryRunner", () => {
  it("reports retrying while the retry command runs", async () => {
    const command = deferred();
    const retrying: boolean[] = [];
    const retry = createRetryRunner(
      () => command.promise,
      (isRetrying) => {
        retrying.push(isRetrying);
      },
    );

    const running = retry();
    expect(retrying).toEqual([true]);

    command.resolve();
    await running;
    expect(retrying).toEqual([true, false]);
  });

  it("ignores a second retry while one runs", async () => {
    const command = deferred();
    let calls = 0;
    const retry = createRetryRunner(
      () => {
        calls += 1;
        return command.promise;
      },
      () => {},
    );

    const running = retry();
    await retry();
    command.resolve();
    await running;

    expect(calls).toBe(1);
  });

  it("allows a new retry once the previous one failed", async () => {
    const command = deferred();
    const retrying: boolean[] = [];
    let calls = 0;
    const retry = createRetryRunner(
      () => {
        calls += 1;
        return calls === 1 ? command.promise : Promise.resolve();
      },
      (isRetrying) => {
        retrying.push(isRetrying);
      },
    );

    const running = retry();
    command.reject(new Error("invoke failed"));
    await expect(running).rejects.toThrow("invoke failed");
    await retry();

    expect(calls).toBe(2);
    expect(retrying).toEqual([true, false, true, false]);
  });
});
