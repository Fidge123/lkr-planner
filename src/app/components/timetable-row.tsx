import { getWorkItemsForCell } from "../../data/dummy-data";
import type { Employee } from "../../domain/planning";
import { isToday } from "../util";
import { TimetableCell } from "./timetable-cell";

export function TimetableRow({ employee, weekDays }: Props) {
  return (
    <tr key={employee.id} className="divide-x divide-slate-300">
      <th className="p-4 align-top font-normal">{employee.name}</th>

      {weekDays.map((day) => (
        <TimetableCell
          key={day.toISOString()}
          highlight={isToday(day)}
          projects={getWorkItemsForCell(employee.id, day)}
        />
      ))}
    </tr>
  );
}

interface Props {
  employee: Employee;
  weekDays: Date[];
}
