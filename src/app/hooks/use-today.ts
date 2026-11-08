import { useSyncExternalStore } from "react";
import { millisecondsUntilNextLocalMidnight, toLocalISODate } from "../util";

const readToday = () => toLocalISODate(new Date());

function subscribeToDayChange(onDayChange: () => void) {
  let timer: ReturnType<typeof setTimeout>;
  const scheduleNextDay = () => {
    timer = setTimeout(() => {
      onDayChange();
      scheduleNextDay();
    }, millisecondsUntilNextLocalMidnight(new Date()));
  };
  scheduleNextDay();
  return () => clearTimeout(timer);
}

/** The local date as `yyyy-MM-dd`, which re-renders its callers at midnight. */
export function useToday(): string {
  return useSyncExternalStore(subscribeToDayChange, readToday);
}
