import { TriangleAlert } from "lucide-react";
import { useState } from "react";
import {
  type CalendarCellEvent,
  commands,
  type EmployeeSetting,
  type PlanningContactRecord,
} from "../../generated/tauri";
import { recordLastAssignedProject } from "../../services/assignment-suggestions";
import type { GhostSuggestion, ModalSaveAction } from "../next-day-quick-add";
import { isGhostVisible, nextGhostState } from "../next-day-quick-add";
import type { CellEvent } from "../types";
import { toCellEvent } from "../types";
import { isToday, toLocalISODate } from "../util";
import { AssignmentModal } from "./assignment-modal";
import { TimetableCell } from "./timetable-cell";

export function TimetableRow({
  employee,
  calendarEvents,
  calendarError,
  week,
  employeeSetting,
  onOpenIcalDialog,
  onReloadAssignments,
}: Props) {
  const showWarning = needsAttention(employeeSetting);
  const [modalState, setModalState] = useState<ModalState | null>(null);
  const [ghost, setGhost] = useState<GhostSuggestion | null>(null);
  const isoWeekDays = week.days.map(toLocalISODate);
  const weekStart = isoWeekDays[0] ?? "";
  const [ghostWeekStart, setGhostWeekStart] = useState(weekStart);

  // A ghost only makes sense within the week it was created for; adjust state
  // during render rather than in an effect to avoid a stale-ghost flash.
  if (weekStart !== ghostWeekStart) {
    setGhostWeekStart(weekStart);
    setGhost(null);
  }

  const openCreateModal = (date: string) =>
    setModalState({ date, assignment: null });

  const openEditModal = (date: string, event: CellEvent) => {
    const source = calendarEvents.find((e) => e.uid === event.uid) ?? null;
    setModalState({ date, assignment: source });
  };

  const handleSave = (action: ModalSaveAction) => {
    setModalState(null);
    setGhost((current) => nextGhostState(current, action, isoWeekDays));
    onReloadAssignments();
  };

  const handleSuggestionClick = async (suggestion: GhostSuggestion) => {
    const result = await commands.createAssignment({
      employeeReference: employee.self,
      date: suggestion.date,
      projectRef: suggestion.projectRef,
      projectName: suggestion.projectName,
    });
    if (result.status === "error") return;
    recordLastAssignedProject({
      self: suggestion.projectRef,
      name: suggestion.projectName,
    });
    setGhost((current) =>
      nextGhostState(
        current,
        {
          kind: "create",
          date: suggestion.date,
          projectRef: suggestion.projectRef,
          projectName: suggestion.projectName,
        },
        isoWeekDays,
      ),
    );
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
          <td colSpan={week.days.length} className="p-4 text-sm align-middle">
            <div className="flex items-center gap-3">
              <span className="text-error" title={calendarError}>
                Kalender nicht verfügbar
              </span>
              <button
                type="button"
                className="btn btn-xs btn-ghost"
                onClick={onReloadAssignments}
              >
                Erneut laden
              </button>
            </div>
          </td>
        ) : (
          week.days.map((day) => {
            const isoDay = toLocalISODate(day);
            const rawDayEvents = calendarEvents.filter(
              (e) => e.date === isoDay,
            );
            const dayEvents = rawDayEvents.map(toCellEvent);
            const suggestion =
              ghost && isGhostVisible(ghost, isoDay, rawDayEvents)
                ? ghost
                : undefined;
            return (
              <TimetableCell
                key={day.toISOString()}
                highlight={isToday(day)}
                isHoliday={week.holidayDates.has(isoDay)}
                events={dayEvents}
                suggestion={suggestion}
                onAddClick={() => openCreateModal(isoDay)}
                onEventClick={(event) => openEditModal(isoDay, event)}
                onSuggestionClick={handleSuggestionClick}
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
  week: {
    days: Date[];
    holidayDates: Set<string>;
  };
  employeeSetting: EmployeeSetting | null;
  onOpenIcalDialog: (employee: PlanningContactRecord) => void;
  onReloadAssignments: () => void;
}
