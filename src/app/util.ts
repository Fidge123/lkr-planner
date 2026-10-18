const millisecondsPerWeek = 7 * 24 * 60 * 60 * 1000;

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

export function weekOffsetFor(day: Date, showWeekend = false): number {
  const daysSinceMonday = (day.getDay() + 6) % 7;
  const monday = new Date(
    day.getFullYear(),
    day.getMonth(),
    day.getDate() - daysSinceMonday,
  );
  const currentMonday = getWeekDays(0, showWeekend)[0];
  return Math.round(
    (monday.getTime() - currentMonday.getTime()) / millisecondsPerWeek,
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
  return (
    1 +
    Math.round(
      (thursday.getTime() - firstThursday.getTime()) / millisecondsPerWeek,
    )
  );
}

export function monthWeeks(year: number, month: number): MonthWeek[] {
  const daysBeforeFirst = (new Date(year, month, 1).getDay() + 6) % 7;
  const daysInMonth = new Date(year, month + 1, 0).getDate();
  return Array.from(
    { length: Math.ceil((daysBeforeFirst + daysInMonth) / 7) },
    (_, row) => {
      const days = Array.from(
        { length: 7 },
        (_, i) => new Date(year, month, 1 - daysBeforeFirst + row * 7 + i),
      );
      return { weekNumber: getIsoWeek(days[0]), days };
    },
  );
}

export interface MonthWeek {
  weekNumber: number;
  days: Date[];
}

// The Thursday's month always holds the whole displayed week together with the month before it.
export function pickerAnchorMonth(weekDays: Date[]): YearMonth {
  const thursday = thursdayOfWeek(weekDays[0]);
  return { year: thursday.getFullYear(), month: thursday.getMonth() };
}

export interface YearMonth {
  year: number;
  month: number;
}

export function shiftMonth({ year, month }: YearMonth, by: number): YearMonth {
  const shifted = new Date(year, month + by, 1);
  return { year: shifted.getFullYear(), month: shifted.getMonth() };
}

function thursdayOfWeek(date: Date): Date {
  const daysSinceMonday = (date.getDay() + 6) % 7;
  return new Date(
    date.getFullYear(),
    date.getMonth(),
    date.getDate() + 3 - daysSinceMonday,
  );
}
