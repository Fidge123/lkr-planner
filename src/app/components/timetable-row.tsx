import { TriangleAlert } from "lucide-react";
import { getWorkItemsForCell } from "../../data/dummy-data";
import type {
  EmployeeSetting,
  PlanningContactRecord,
  PlanningProjectRecord,
} from "../../generated/tauri";
import { isToday } from "../util";
import { TimetableCell } from "./timetable-cell";

export function TimetableRow({
  employee,
  projects,
  weekDays,
  employeeSetting,
  onOpenIcalDialog,
}: Props) {
  const showWarning = needsAttention(employeeSetting);

  return (
    <tr key={employee.self}>
      <th className="p-0 align-top font-normal">
        <button
          type="button"
          className="w-full h-full p-4 text-left flex items-center justify-between gap-2 hover:bg-base-200 cursor-pointer"
          onClick={() => onOpenIcalDialog(employee)}
        >
          <span>
            {employee.nickname ?? employee.full_name ?? "Unbenannter Kontakt"}
          </span>
          {showWarning ? (
            <TriangleAlert
              className="size-4 text-warning shrink-0 mt-0.5"
              aria-label="Kalender-Verbindung nicht bestätigt"
            />
          ) : null}
        </button>
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

/** Returns true when the primary iCal source needs attention (no calendar, untested, or last test failed). */
function needsAttention(setting: EmployeeSetting | null | undefined): boolean {
  if (!setting) {
    return true;
  }
  if (!setting.zepPrimaryCalendar) {
    return true;
  }
  if (!setting.primaryIcalLastTestedAt) {
    return true;
  }
  if (setting.primaryIcalLastTestPassed === false) {
    return true;
  }
  return false;
}

interface Props {
  employee: PlanningContactRecord;
  projects: PlanningProjectRecord[];
  weekDays: Date[];
  employeeSetting: EmployeeSetting | null;
  onOpenIcalDialog: (employee: PlanningContactRecord) => void;
}
