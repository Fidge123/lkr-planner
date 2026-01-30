export function getWeekDays(weekOffset: number) {
  const today = new Date();
  const currentDay = today.getDay();
  const dayInMs = 1000 * 60 * 60 * 24;
  // Calculate Monday of the current week (or next week if today is weekend)
  const mondayOffset =
    currentDay === 0 ? 1 : currentDay === 6 ? 2 : 1 - currentDay;
  const monday = new Date(today.toISOString().slice(0, 10));
  monday.setDate(today.getDate() + mondayOffset + weekOffset * 7);

  return [
    monday,
    new Date(monday.getTime() + dayInMs),
    new Date(monday.getTime() + dayInMs * 2),
    new Date(monday.getTime() + dayInMs * 3),
    new Date(monday.getTime() + dayInMs * 4),
  ];
}

export function isToday(day: Date) {
  const today = new Date();
  return day.toDateString() === today.toDateString();
}
