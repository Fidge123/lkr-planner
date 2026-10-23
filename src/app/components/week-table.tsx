import { CalendarDays } from "lucide-react";
import { useEffect, useRef } from "react";
import type {
  CalendarCellEvent,
  EmployeeSetting,
  Holiday,
  PlanningContactRecord,
} from "../../generated/tauri";
import type { ProjectCategoryColors } from "../../services/daylite-categories";
import type { DropPreview } from "../hooks/use-appointment-drag";
import type { CellEvent } from "../types";
import { getIsoWeek, toLocalISODate } from "../util";
import { stickyHeaderClass, TimetableHeader } from "./timetable-header";
import { TimetableRow } from "./timetable-row";
import { WeekPicker } from "./week-picker";

export function WeekTable({
  weekDays,
  employees,
  employeeSettings,
  eventsByEmployee,
  errorsByEmployee,
  categoryColors,
  holidays,
  isEmployeeLoading,
  dropPreview = null,
  draggedUid = null,
  activeHighlight = null,
  onOpenIcalDialog,
  onToggleHighlight,
  onJumpToWeek,
}: WeekTableProps) {
  const holidayByDate = new Map(holidays.map((h) => [h.date, h.name]));
  const holidayDates = new Set(holidays.map((h) => h.date));

  // WKWebView keeps a re-slotted card's stale pixels after a cell reload.
  // Promoting the grid to a compositing layer for one frame forces a full repaint.
  const gridRef = useRef<HTMLTableElement>(null);
  // biome-ignore lint/correctness/useExhaustiveDependencies: eventsByEmployee is the repaint trigger, not a value the effect body reads.
  useEffect(() => {
    const grid = gridRef.current;
    if (!grid) return;
    grid.style.transform = "translateZ(0)";
    const frame = requestAnimationFrame(() => {
      grid.style.transform = "";
    });
    return () => cancelAnimationFrame(frame);
  }, [eventsByEmployee]);

  const calendarWeek = (
    <>
      <CalendarDays className="size-4" aria-hidden="true" />
      KW {getIsoWeek(weekDays[0])}
    </>
  );

  return (
    // A card the pointer crosses mid-drag, or that an autoscroll slides under it, otherwise keeps its hover for good.
    // dnd-kit measures rects rather than hit-testing, so it loses nothing by this.
    <table
      ref={gridRef}
      className={`table table-fixed border-collapse ${draggedUid ? "pointer-events-none" : ""}`}
    >
      <thead className="text-base-content">
        <tr>
          <th className={`w-40 p-4 font-bold bg-base-100 ${stickyHeaderClass}`}>
            {onJumpToWeek ? (
              <WeekPicker weekDays={weekDays} onJumpToWeek={onJumpToWeek}>
                {calendarWeek}
              </WeekPicker>
            ) : (
              <span className="flex items-center gap-1.5 text-primary">
                {calendarWeek}
              </span>
            )}
          </th>
          {weekDays.map((day) => (
            <TimetableHeader
              key={day.getTime()}
              day={day}
              holiday={holidayByDate.get(toLocalISODate(day))}
            />
          ))}
        </tr>
      </thead>
      <tbody>
        {employees.map((employee, index) => (
          <TimetableRow
            key={employee.self || `employee-${index}`}
            employee={employee}
            calendarEvents={eventsByEmployee[employee.self] ?? []}
            calendarError={errorsByEmployee[employee.self] ?? null}
            categoryColors={categoryColors}
            week={{ days: weekDays, holidayDates }}
            employeeSetting={
              employeeSettings.find(
                (s) => s.dayliteContactReference === employee.self,
              ) ?? null
            }
            dropPreview={dropPreview}
            draggedUid={draggedUid}
            activeHighlight={activeHighlight}
            onOpenIcalDialog={onOpenIcalDialog}
            onToggleHighlight={onToggleHighlight}
          />
        ))}
        {!isEmployeeLoading && employees.length === 0 ? (
          <tr key="no-employees-row">
            <td
              className="p-4 text-base-content/70"
              colSpan={weekDays.length + 1}
            >
              Keine Mitarbeiter gefunden
            </td>
          </tr>
        ) : null}
      </tbody>
    </table>
  );
}

export interface WeekTableProps {
  weekDays: Date[];
  employees: PlanningContactRecord[];
  employeeSettings: EmployeeSetting[];
  eventsByEmployee: Record<string, CalendarCellEvent[]>;
  errorsByEmployee: Record<string, string>;
  categoryColors: ProjectCategoryColors;
  holidays: Holiday[];
  isEmployeeLoading: boolean;
  dropPreview?: DropPreview | null;
  draggedUid?: string | null;
  activeHighlight?: string | null;
  onOpenIcalDialog: (employee: PlanningContactRecord) => void;
  onToggleHighlight?: (event: CellEvent) => void;
  onJumpToWeek?: (day: Date) => void;
}
