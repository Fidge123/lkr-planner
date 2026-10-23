import { useEffect, useRef, useState } from "react";
import { scheduleTimeout } from "../util";

const autoRefreshIntervalMs = 5 * 60_000;
// A Daylite token rotation lost on a network still connecting after wake signs the user out.
const returnSettleDelayMs = 5_000;

export function useCalendarAutoRefresh(
  lastLoadedAt: number | null,
  refresh: () => void,
): void {
  const refreshRef = useRef(refresh);
  refreshRef.current = refresh;
  const [scheduler] = useState(() =>
    createAutoRefreshScheduler({
      now: Date.now,
      schedule: scheduleTimeout,
      isVisible: () => document.visibilityState === "visible",
      isOnline: () => navigator.onLine,
      refresh: () => refreshRef.current(),
    }),
  );

  useEffect(() => {
    scheduler.setLastLoadedAt(lastLoadedAt);
  }, [scheduler, lastLoadedAt]);

  useEffect(() => {
    document.addEventListener("visibilitychange", scheduler.onReturn);
    window.addEventListener("focus", scheduler.onReturn);
    window.addEventListener("online", scheduler.onReturn);
    return () => {
      document.removeEventListener("visibilitychange", scheduler.onReturn);
      window.removeEventListener("focus", scheduler.onReturn);
      window.removeEventListener("online", scheduler.onReturn);
      scheduler.dispose();
    };
  }, [scheduler]);
}

export interface AutoRefreshDeps {
  now: () => number;
  schedule: (callback: () => void, ms: number) => () => void;
  isVisible: () => boolean;
  isOnline: () => boolean;
  refresh: () => void;
}

/** Hidden windows throttle timers and sleep freezes them, so a return re-checks the data's age. */
export function createAutoRefreshScheduler({
  now,
  schedule,
  isVisible,
  isOnline,
  refresh,
}: AutoRefreshDeps) {
  let lastLoadedAt: number | null = null;
  let cancelDue = () => {};
  let cancelCheck = () => {};

  const refreshIfAllowed = () => {
    if (!isVisible() || !isOnline()) return;
    cancelCheck();
    refresh();
  };

  return {
    setLastLoadedAt: (loadedAt: number | null) => {
      lastLoadedAt = loadedAt;
      cancelDue();
      cancelDue =
        loadedAt === null
          ? () => {}
          : schedule(
              refreshIfAllowed,
              Math.max(0, loadedAt + autoRefreshIntervalMs - now()),
            );
    },
    onReturn: () => {
      cancelCheck();
      cancelCheck = schedule(() => {
        if (
          lastLoadedAt !== null &&
          now() - lastLoadedAt >= autoRefreshIntervalMs
        ) {
          refreshIfAllowed();
        }
      }, returnSettleDelayMs);
    },
    dispose: () => {
      cancelDue();
      cancelCheck();
    },
  };
}
