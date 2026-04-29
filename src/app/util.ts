export function getWeekDays(weekOffset: number) {
  const today = new Date();
  const currentDay = today.getDay();
  // Calculate Monday of the current week (or next week if today is weekend)
  const mondayOffset =
    currentDay === 0 ? 1 : currentDay === 6 ? 2 : 1 - currentDay;
  const mondayDate = today.getDate() + mondayOffset + weekOffset * 7;

  // Use local-time date construction to avoid UTC/DST offset issues.
  return Array.from(
    { length: 5 },
    (_, i) => new Date(today.getFullYear(), today.getMonth(), mondayDate + i),
  );
}

export function isToday(day: Date) {
  const today = new Date();
  return day.toDateString() === today.toDateString();
}

/** Formats a Date as "yyyy-MM-dd" using local time components (not UTC). */
export function toLocalISODate(date: Date): string {
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, "0");
  const d = String(date.getDate()).padStart(2, "0");
  return `${y}-${m}-${d}`;
}
