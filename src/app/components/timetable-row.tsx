import { getWorkItemsForCell } from "../../data/dummy-data";
import {
  type DayliteContactRecord,
  getDayliteContactDisplayName,
} from "../../domain/planning";
import { isToday } from "../util";
import { TimetableCell } from "./timetable-cell";

export function TimetableRow({ employee, weekDays }: Props) {
  return (
    <tr key={employee.self} className="divide-x divide-slate-300">
      <th className="p-4 align-top font-normal">
        {getDayliteContactDisplayName(employee)}
      </th>

      {weekDays.map((day) => (
        <TimetableCell
          key={day.toISOString()}
          highlight={isToday(day)}
          projects={getWorkItemsForCell(employee.self, day)}
        />
      ))}
    </tr>
  );
}

interface Props {
  employee: DayliteContactRecord;
  weekDays: Date[];
}
