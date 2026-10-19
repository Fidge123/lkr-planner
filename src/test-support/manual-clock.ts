export function manualClock(start = 0) {
  let time = start;
  let timers: { at: number; callback: () => void }[] = [];

  return {
    now: () => time,
    schedule: (callback: () => void, ms: number) => {
      const timer = { at: time + ms, callback };
      timers.push(timer);
      return () => {
        timers = timers.filter((candidate) => candidate !== timer);
      };
    },
    advance: (ms: number) => {
      const until = time + ms;
      for (;;) {
        const next = timers
          .filter((timer) => timer.at <= until)
          .sort((a, b) => a.at - b.at)[0];
        if (!next) break;
        timers = timers.filter((timer) => timer !== next);
        time = Math.max(time, next.at);
        next.callback();
      }
      time = until;
    },
    pendingTimers: () => timers.length,
  };
}
