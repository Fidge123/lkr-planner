import { useEffect, useState } from "react";
import { millisecondsUntilNextLocalMidnight, toLocalISODate } from "../util";

export function useRerenderOnDayChange(): void {
  const [, setToday] = useState(() => toLocalISODate(new Date()));

  useEffect(() => {
    let timer: ReturnType<typeof setTimeout>;

    const scheduleNextDay = () => {
      const now = new Date();
      timer = setTimeout(() => {
        setToday(toLocalISODate(new Date()));
        scheduleNextDay();
      }, millisecondsUntilNextLocalMidnight(now));
    };

    scheduleNextDay();
    return () => clearTimeout(timer);
  }, []);
}
