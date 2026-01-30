import { isToday } from "../util";

export function TimetableHeader({ day }: Props) {
  return (
    <th
      key={day.getTime()}
      className={`text-center ${
        isToday(day) ? "bg-primary text-primary-content" : ""
      }`}
    >
      <time dateTime={day.toISOString()}>
        {day.toLocaleDateString("de-DE", {
          weekday: "long",
          day: "2-digit",
          month: "2-digit",
        })}
      </time>
    </th>
  );
}

interface Props {
  day: Date;
}
