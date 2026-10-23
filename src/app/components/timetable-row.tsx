import { TriangleAlert } from "lucide-react";
import { useState } from "react";
import { createPortal } from "react-dom";
import type {
  CalendarCellEvent,
  EmployeeSetting,
  PlanningContactRecord,
} from "../../generated/tauri";
import { recordLastAssignedProject } from "../../services/assignment-suggestions";
import { createAssignment } from "../../services/assignments";
import type { ProjectCategoryColors } from "../../services/daylite-categories";
import type { DropPreview } from "../hooks/use-appointment-drag";
import { useWeekCache } from "../hooks/use-planning-assignments";
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
  categoryColors,
  week,
  employeeSetting,
  dropPreview = null,
  draggedUid = null,
  activeHighlight = null,
  onOpenIcalDialog,
  onToggleHighlight,
}: Props) {
  const showWarning = needsAttention(employeeSetting);
  const [modalState, setModalState] = useState<ModalState | null>(null);
  const [ghost, setGhost] = useState<GhostSuggestion | null>(null);
  const isoWeekDays = week.days.map(toLocalISODate);
  const weekStart = isoWeekDays[0] ?? "";
  const [ghostWeekStart, setGhostWeekStart] = useState(weekStart);

  if (weekStart !== ghostWeekStart) {
    setGhostWeekStart(weekStart);
    setGhost(null);
  }

  const cache = useWeekCache();
  const handleWritten = (removedUid: string | null) => {
    if (removedUid !== null) cache.removeEvent(removedUid);
    void cache.refreshEmployees(weekStart, [employee.self]);
  };

  const openCreateModal = (date: string) =>
    setModalState({ date, assignment: null });

  const openEditModal = (date: string, event: CellEvent) => {
    const source = calendarEvents.find((e) => e.uid === event.uid) ?? null;
    setModalState({ date, assignment: source });
  };

  const handleSave = (action: ModalSaveAction) => {
    const removedUid =
      action.kind === "delete" ? (modalState?.assignment?.uid ?? null) : null;
    setModalState(null);
    setGhost((current) => nextGhostState(current, action, isoWeekDays));
    handleWritten(removedUid);
  };

  const handleSuggestionClick = async (suggestion: GhostSuggestion) => {
    try {
      await createAssignment({
        employeeReference: employee.self,
        date: suggestion.date,
        projectRef: suggestion.projectRef,
        projectName: suggestion.projectName,
      });
    } catch {
      return;
    }
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
    handleWritten(null);
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
                onClick={() => void cache.reload(weekStart)}
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
            const dayEvents = rawDayEvents.map((event) =>
              toCellEvent(event, categoryColors),
            );
            const suggestion =
              ghost && isGhostVisible(ghost, isoDay, rawDayEvents)
                ? ghost
                : undefined;
            return (
              <TimetableCell
                key={day.toISOString()}
                isToday={isToday(day)}
                isHoliday={week.holidayDates.has(isoDay)}
                employeeRef={employee.self}
                date={isoDay}
                events={dayEvents}
                suggestion={suggestion}
                dropPreview={dropPreview}
                draggedUid={draggedUid}
                activeHighlight={activeHighlight}
                onAddClick={() => openCreateModal(isoDay)}
                onEventClick={(event) => openEditModal(isoDay, event)}
                onSuggestionClick={handleSuggestionClick}
                onToggleHighlight={onToggleHighlight}
              />
            );
          })
        )}
      </tr>

      {/* The grid's scroll container is isolated, so a modal inside it could not paint over the header. */}
      {/* The document guard is for bun specs, which render this row with react-dom/server. */}
      {modalState === null || typeof document === "undefined"
        ? null
        : createPortal(
            <AssignmentModal
              assignment={modalState.assignment}
              employeeReference={employee.self}
              date={modalState.date}
              onSave={handleSave}
              onClose={() => setModalState(null)}
            />,
            document.body,
          )}
    </>
  );
}

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
  categoryColors: ProjectCategoryColors;
  week: {
    days: Date[];
    holidayDates: Set<string>;
  };
  employeeSetting: EmployeeSetting | null;
  dropPreview?: DropPreview | null;
  draggedUid?: string | null;
  activeHighlight?: string | null;
  onOpenIcalDialog: (employee: PlanningContactRecord) => void;
  /** A deleted assignment's UID lets the card disappear before the row is refreshed. */
  onToggleHighlight?: (event: CellEvent) => void;
}
