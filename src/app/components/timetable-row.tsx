import { getWorkItemsForCell } from "../../data/dummy-data";
import type {
  PlanningContactRecord,
  PlanningProjectRecord,
} from "../../generated/tauri";
import { isToday } from "../util";
import { TimetableCell } from "./timetable-cell";

export function TimetableRow({ employee, projects, weekDays }: Props) {
  return (
    <tr key={employee.self}>
      <th className="p-4 align-top font-normal">
        {employee.nickname ?? employee.full_name ?? "Unbenannter Kontakt"}
      </th>

      {weekDays.map((day) => (
        <TimetableCell
          key={day.toISOString()}
          highlight={isToday(day)}
          projects={getWorkItemsForCell(employee.self, day, projects)}
        />
      ))}
    </tr>
  );
}

interface Props {
  employee: PlanningContactRecord;
  projects: PlanningProjectRecord[];
  weekDays: Date[];
}
