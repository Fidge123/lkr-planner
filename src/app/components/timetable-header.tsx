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
        <p className="text-xs font-normal mt-0.5">{holiday}</p>
      ) : null}
    </th>
  );
}

function headerClass(day: Date, isHoliday: boolean): string {
  if (isToday(day)) return "text-center bg-primary text-primary-content";
  if (isHoliday) return "text-center text-base-content/50";
  return "text-center";
}

interface Props {
  day: Date;
  holiday?: string;
}
