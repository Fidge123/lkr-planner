import { toLocalISODate } from "../util";

export function TimetableHeader({ day, isToday, holiday }: Props) {
  const isHoliday = Boolean(holiday);
  return (
    <th key={day.getTime()} className={headerClass(isToday, isHoliday)}>
      <time dateTime={toLocalISODate(day)}>
        {day.toLocaleDateString("de-DE", {
          weekday: "long",
          day: "2-digit",
          month: "2-digit",
        })}
      </time>
      {isHoliday ? (
        <small className="block text-xs font-normal mt-0.5">{holiday}</small>
      ) : null}
    </th>
  );
}

// A collapsed table paints its borders itself, so the shadow redraws the bottom border the pinned row leaves behind.
export const stickyHeaderClass =
  "sticky top-0 z-10 shadow-[0_1px_0_0_var(--color-base-300)]";

function headerClass(isToday: boolean, isHoliday: boolean): string {
  const sticky = `text-center ${stickyHeaderClass}`;
  if (isToday) return `${sticky} bg-primary text-primary-content`;
  if (isHoliday) return `${sticky} bg-base-100 text-base-content/50`;
  return `${sticky} bg-base-100`;
}

interface Props {
  day: Date;
  isToday: boolean;
  holiday?: string;
}
