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

export function isToday(day: Date) {
  const today = new Date();
  return day.toDateString() === today.toDateString();
}

export function toLocalISODate(date: Date): string {
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, "0");
  const d = String(date.getDate()).padStart(2, "0");
  return `${y}-${m}-${d}`;
}
