import { describe, expect, it } from "bun:test";
import { manualClock } from "../../test-support/manual-clock";
import { createAutoRefreshScheduler } from "./use-calendar-auto-refresh";

const minutes = (count: number) => count * 60_000;

function scheduler(start = 0) {
  const clock = manualClock(start);
  const environment = { visible: true, online: true, refreshes: 0 };
  const instance = createAutoRefreshScheduler({
    now: clock.now,
    schedule: clock.schedule,
    isVisible: () => environment.visible,
    isOnline: () => environment.online,
    refresh: () => {
      environment.refreshes += 1;
    },
  });
  return { clock, environment, scheduler: instance };
}

describe("createAutoRefreshScheduler", () => {
  it("refreshes 5 minutes after the last load", () => {
    const { clock, environment, scheduler: auto } = scheduler();

    auto.setLastLoadedAt(0);
    clock.advance(minutes(5) - 1);
    expect(environment.refreshes).toBe(0);
    clock.advance(1);

    expect(environment.refreshes).toBe(1);
  });

  it("counts the interval from the time the shown data was loaded", () => {
    const { clock, environment, scheduler: auto } = scheduler(minutes(3));

    auto.setLastLoadedAt(0);
    clock.advance(minutes(2) - 1);
    expect(environment.refreshes).toBe(0);
    clock.advance(1);

    expect(environment.refreshes).toBe(1);
  });

  it("restarts the interval when the last load changes", () => {
    const { clock, environment, scheduler: auto } = scheduler();
    auto.setLastLoadedAt(0);
    clock.advance(minutes(4));

    auto.setLastLoadedAt(minutes(4));
    clock.advance(minutes(5) - 1);
    expect(environment.refreshes).toBe(0);
    clock.advance(1);

    expect(environment.refreshes).toBe(1);
  });

  it("skips a due refresh while the window is hidden", () => {
    const { clock, environment, scheduler: auto } = scheduler();
    environment.visible = false;

    auto.setLastLoadedAt(0);
    clock.advance(minutes(10));

    expect(environment.refreshes).toBe(0);
  });

  it("skips a due refresh while offline", () => {
    const { clock, environment, scheduler: auto } = scheduler();
    environment.online = false;

    auto.setLastLoadedAt(0);
    clock.advance(minutes(10));

    expect(environment.refreshes).toBe(0);
  });

  it("refreshes 5 seconds after a return when the refresh is due", () => {
    const { clock, environment, scheduler: auto } = scheduler();
    environment.visible = false;
    auto.setLastLoadedAt(0);
    clock.advance(minutes(10));

    environment.visible = true;
    auto.onReturn();
    clock.advance(4_999);
    expect(environment.refreshes).toBe(0);
    clock.advance(1);

    expect(environment.refreshes).toBe(1);
  });

  it("does not refresh on a return when the refresh is not due", () => {
    const { clock, environment, scheduler: auto } = scheduler();
    auto.setLastLoadedAt(0);
    clock.advance(minutes(1));

    auto.onReturn();
    clock.advance(5_000);

    expect(environment.refreshes).toBe(0);
  });

  it("checks once 5 seconds after the last of several return events", () => {
    const { clock, environment, scheduler: auto } = scheduler();
    environment.online = false;
    auto.setLastLoadedAt(0);
    clock.advance(minutes(10));
    environment.online = true;

    auto.onReturn();
    clock.advance(3_000);
    auto.onReturn();
    clock.advance(4_999);
    expect(environment.refreshes).toBe(0);
    clock.advance(1);

    expect(environment.refreshes).toBe(1);
  });
});
