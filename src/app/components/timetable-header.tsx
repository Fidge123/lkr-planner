import { isToday } from "../util";

export function TimetableHeader({ day, holiday }: Props) {
  const isHoliday = Boolean(holiday);
  return (
    <th key={day.getTime()} className={headerClass(day, isHoliday)}>
      <time dateTime={day.toISOString()}>
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

function headerClass(day: Date, isHoliday: boolean): string {
  const today = isToday(day);
  if (today && isHoliday)
    return "text-center border-l-2 border-r-2 border-t-2 border-primary text-base-content/50";
  if (today) return "text-center bg-primary text-primary-content";
  if (isHoliday) return "text-center text-base-content/50";
  return "text-center";
}

interface Props {
  day: Date;
  holiday?: string;
}
