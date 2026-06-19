export function getWeekDays(weekOffset: number, showWeekend = false) {
  const today = new Date();
  const currentDay = today.getDay();
  // Calculate Monday of the displayed week. With weekends hidden, a weekend day
  // anchors to the upcoming Monday (the present weekend day is not shown anyway).
  // With weekends shown, anchor to the Monday of the week containing today so
  // today's Saturday or Sunday stays visible at weekOffset 0.
  let mondayOffset: number;
  if (currentDay === 0) {
    // Sunday
    mondayOffset = showWeekend ? -6 : 1;
  } else if (currentDay === 6) {
    // Saturday
    mondayOffset = showWeekend ? -5 : 2;
  } else {
    // Monday to Friday
    mondayOffset = 1 - currentDay;
  }
  const mondayDate = today.getDate() + mondayOffset + weekOffset * 7;

  // Use local-time date construction to avoid UTC/DST offset issues.
  return Array.from(
    { length: showWeekend ? 7 : 5 },
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
