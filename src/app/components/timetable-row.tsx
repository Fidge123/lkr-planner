import { TriangleAlert } from "lucide-react";
import { useState } from "react";
import type {
  CalendarCellEvent,
  EmployeeSetting,
  PlanningContactRecord,
} from "../../generated/tauri";
import type { CellEvent } from "../types";
import { toCellEvent } from "../types";
import { isToday, toLocalISODate } from "../util";
import { AssignmentModal } from "./assignment-modal";
import { TimetableCell } from "./timetable-cell";

export function TimetableRow({
  employee,
  calendarEvents,
  calendarError,
  weekDays,
  employeeSetting,
  holidayDates,
  onRetry,
  onOpenIcalDialog,
  onReloadAssignments,
}: Props) {
  const showWarning = needsAttention(employeeSetting);
  const [modalState, setModalState] = useState<ModalState | null>(null);

  const openCreateModal = (date: string) =>
    setModalState({ date, assignment: null });

  const openEditModal = (date: string, event: CellEvent) => {
    const source = calendarEvents.find((e) => e.uid === event.uid) ?? null;
    setModalState({ date, assignment: source });
  };

  const handleSave = () => {
    setModalState(null);
    onReloadAssignments();
  };

  return (
    <>
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

        {calendarError ? (
          <td colSpan={weekDays.length} className="p-4 text-sm align-middle">
            <div className="flex items-center gap-3">
              <span className="text-error" title={calendarError}>
                Kalender nicht verfügbar
              </span>
              <button
                type="button"
                className="btn btn-xs btn-ghost"
                onClick={onRetry}
              >
                Erneut laden
              </button>
            </div>
          </td>
        ) : (
          weekDays.map((day) => {
            const isoDay = toLocalISODate(day);
            const dayEvents = calendarEvents
              .filter((e) => e.date === isoDay)
              .map(toCellEvent);
            return (
              <TimetableCell
                key={day.toISOString()}
                highlight={isToday(day)}
                isHoliday={holidayDates.has(isoDay)}
                events={dayEvents}
                onAddClick={() => openCreateModal(isoDay)}
                onEventClick={(event) => openEditModal(isoDay, event)}
              />
            );
          })
        )}
      </tr>

      <AssignmentModal
        isOpen={modalState !== null}
        assignment={modalState?.assignment ?? null}
        employeeReference={employee.self}
        date={modalState?.date ?? ""}
        onSave={handleSave}
        onClose={() => setModalState(null)}
      />
    </>
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

interface ModalState {
  date: string;
  assignment: CalendarCellEvent | null;
}

interface Props {
  employee: PlanningContactRecord;
  calendarEvents: CalendarCellEvent[];
  calendarError: string | null;
  weekDays: Date[];
  employeeSetting: EmployeeSetting | null;
  holidayDates: Set<string>;
  onRetry: () => void;
  onOpenIcalDialog: (employee: PlanningContactRecord) => void;
  onReloadAssignments: () => void;
}
