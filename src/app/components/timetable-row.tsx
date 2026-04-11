import { TriangleAlert } from "lucide-react";
import type {
  CalendarCellEvent,
  EmployeeSetting,
  PlanningContactRecord,
} from "../../generated/tauri";
import { toCellEvent } from "../types";
import { isToday } from "../util";
import { TimetableCell } from "./timetable-cell";

export function TimetableRow({
  employee,
  calendarEvents,
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

      {weekDays.map((day) => {
        const isoDay = day.toISOString().slice(0, 10);
        const dayEvents = calendarEvents
          .filter((e) => e.date === isoDay)
          .map(toCellEvent);
        return (
          <TimetableCell
            key={day.toISOString()}
            highlight={isToday(day)}
            events={dayEvents}
          />
        );
      })}
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
  calendarEvents: CalendarCellEvent[];
  weekDays: Date[];
  employeeSetting: EmployeeSetting | null;
  onOpenIcalDialog: (employee: PlanningContactRecord) => void;
}
