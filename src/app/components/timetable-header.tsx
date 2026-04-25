import { isToday } from "../util";

export function TimetableHeader({ day, holiday }: Props) {
  const isHoliday = holiday !== undefined;
  return (
    <th
      key={day.getTime()}
      className={`text-center ${
        isToday(day)
          ? "bg-primary text-primary-content"
          : isHoliday
            ? "text-base-content/50"
            : ""
      }`}
    >
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

interface Props {
  day: Date;
  holiday?: string;
}
