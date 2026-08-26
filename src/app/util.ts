export function getWeekDays(weekOffset: number, showWeekend = false) {
  const today = new Date();
  const currentDay = today.getDay();
  let mondayOffset: number;
  if (currentDay === 0) {
    mondayOffset = showWeekend ? -6 : 1;
  } else if (currentDay === 6) {
    mondayOffset = showWeekend ? -5 : 2;
  } else {
    mondayOffset = 1 - currentDay;
  }
  const mondayDate = today.getDate() + mondayOffset + weekOffset * 7;

  return Array.from(
    { length: showWeekend ? 7 : 5 },
    (_, i) => new Date(today.getFullYear(), today.getMonth(), mondayDate + i),
  );
}

export function shiftWeekDays(days: Date[], weekOffset: number): Date[] {
  return days.map(
    (day) =>
      new Date(
        day.getFullYear(),
        day.getMonth(),
        day.getDate() + weekOffset * 7,
      ),
  );
}

export function millisecondsUntilNextLocalMidnight(now: Date): number {
  const nextMidnight = new Date(
    now.getFullYear(),
    now.getMonth(),
    now.getDate() + 1,
  );
  return nextMidnight.getTime() - now.getTime();
}

export function isToday(day: Date) {
  const today = new Date();
  return day.toDateString() === today.toDateString();
}

// Must stay local-time based: the backend fetches [weekStart, weekStart + 7d),
// so a UTC-shifted start would drop the last displayed day east of UTC.
export function getWeekStart(weekOffset: number, showWeekend = false): string {
  return toLocalISODate(getWeekDays(weekOffset, showWeekend)[0]);
}

export function toLocalISODate(date: Date): string {
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, "0");
  const d = String(date.getDate()).padStart(2, "0");
  return `${y}-${m}-${d}`;
}

// ISO 8601: the week of a date is the week holding the Thursday of that date's week, and week 1 is the one holding 4 January.
export function getIsoWeek(date: Date): number {
  const thursday = thursdayOfWeek(date);
  const firstThursday = thursdayOfWeek(new Date(thursday.getFullYear(), 0, 4));
  const millisecondsPerWeek = 7 * 24 * 60 * 60 * 1000;
  return (
    1 +
    Math.round(
      (thursday.getTime() - firstThursday.getTime()) / millisecondsPerWeek,
    )
  );
}

function thursdayOfWeek(date: Date): Date {
  const daysSinceMonday = (date.getDay() + 6) % 7;
  return new Date(
    date.getFullYear(),
    date.getMonth(),
    date.getDate() + 3 - daysSinceMonday,
  );
}
