import { employees, getWorkItemsForCell } from "../data/dummyData";
import type { WeekDay } from "../types";
import { WorkItemBadge } from "./WorkItemBadge";

interface PlanningGridProps {
  weekOffset: number;
}

function getWeekDays(weekOffset: number): (WeekDay & { isToday: boolean })[] {
  const today = new Date();
  const currentDay = today.getDay();
  // Calculate Monday of the current week (or next week if today is weekend)
  const mondayOffset =
    currentDay === 0 ? 1 : currentDay === 6 ? 2 : 1 - currentDay;
  const monday = new Date(today);
  monday.setDate(today.getDate() + mondayOffset + weekOffset * 7);

  const weekDays: (WeekDay & { isToday: boolean })[] = [];
  const dayNames = ["Montag", "Dienstag", "Mittwoch", "Donnerstag", "Freitag"];
  const shortNames = ["Mo", "Di", "Mi", "Do", "Fr"];

  for (let i = 0; i < 5; i++) {
    const date = new Date(monday);
    date.setDate(monday.getDate() + i);
    const isToday =
      date.getDate() === today.getDate() &&
      date.getMonth() === today.getMonth() &&
      date.getFullYear() === today.getFullYear();
    weekDays.push({
      index: i,
      name: dayNames[i],
      shortName: shortNames[i],
      date,
      isToday,
    });
  }

  return weekDays;
}

export function PlanningGrid({ weekOffset }: PlanningGridProps) {
  const weekDays = getWeekDays(weekOffset);

  return (
    <section className="w-full h-full overflow-auto p-4">
      <table className="table table-fixed min-w-[900px] border-collapse">
        <thead>
          <tr>
            <th className="w-[200px] p-[2px]">
              <div className="w-full h-full bg-base-200 rounded-lg p-3 flex items-center justify-center">
                <span className="text-sm font-semibold text-base-content/70">
                  Mitarbeiter
                </span>
              </div>
            </th>
            {weekDays.map((day) => (
              <th
                key={day.index}
                className={`p-[2px] ${day.isToday ? "py-0 align-bottom" : "align-top"}`}
              >
                <div
                  className={`w-full h-full p-3 text-center flex flex-col justify-center ${
                    day.isToday
                      ? "bg-primary text-primary-content rounded-t-lg"
                      : "bg-base-200 rounded-lg"
                  }`}
                >
                  <strong className={day.isToday ? "" : "text-base-content"}>
                    {day.shortName}
                  </strong>
                  <time
                    dateTime={day.date.toISOString().split("T")[0]}
                    className={`block text-xs ${day.isToday ? "text-primary-content/80" : "text-base-content/60"}`}
                  >
                    {day.date.toLocaleDateString("de-DE", {
                      day: "2-digit",
                      month: "2-digit",
                    })}
                  </time>
                </div>
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {employees.map((employee, empIndex) => (
            <tr key={employee.id}>
              <th className="p-[2px] align-top font-normal">
                <div
                  className={`w-full h-full p-3 flex items-center gap-3 border-r border-base-300 rounded-lg text-left ${
                    empIndex % 2 === 0 ? "bg-base-100" : "bg-base-200/30"
                  }`}
                >
                  <address className="not-italic flex-1 min-w-0">
                    <strong className="font-semibold text-sm block truncate">
                      {employee.name}
                    </strong>
                    <small className="text-xs text-base-content/60 truncate block">
                      {employee.role}
                    </small>
                  </address>
                </div>
              </th>

              {weekDays.map((day) => {
                const cellWorkItems = getWorkItemsForCell(
                  employee.id,
                  day.index,
                );
                const displayItems = cellWorkItems.slice(0, 3);
                const hasOverflow = cellWorkItems.length > 3;

                const isFirstEmployee = empIndex === 0;
                const isLastEmployee = empIndex === employees.length - 1;

                // Determine border classes for 'today' column to merge cells visually
                const todayBorderClass = day.isToday
                  ? `border-x border-primary/30 ${isFirstEmployee ? "border-t" : "border-t-0"} ${isLastEmployee ? "border-b" : "border-b-0"}`
                  : "border border-base-300/50";

                const todayRoundedClass = day.isToday
                  ? `${isLastEmployee ? "rounded-b-lg" : "rounded-none"}`
                  : "rounded-lg";

                return (
                  <td
                    key={`${employee.id}-${day.index}`}
                    className={`p-[2px] align-top ${day.isToday ? "py-0" : ""}`}
                  >
                    <div
                      className={`w-full h-full p-2 min-h-[100px] transition-colors ${todayBorderClass} ${todayRoundedClass} ${
                        day.isToday
                          ? "bg-primary/5 hover:bg-primary/10"
                          : "bg-base-100 hover:border-primary/30 hover:bg-base-200/20"
                      }`}
                    >
                      <ul className="flex flex-col gap-1 h-full list-none p-0 m-0">
                        {displayItems.map((workItem) => (
                          <li key={workItem.id}>
                            <WorkItemBadge
                              workItem={workItem}
                              dayIndex={day.index}
                              currentEmployeeId={employee.id}
                            />
                          </li>
                        ))}

                        {hasOverflow && (
                          <li className="text-xs text-base-content/50 text-center mt-auto">
                            +{cellWorkItems.length - 3} weitere
                          </li>
                        )}

                        {displayItems.length === 0 && (
                          <li className="flex-1 flex items-center justify-center">
                            <button
                              type="button"
                              className="btn btn-ghost btn-circle btn-sm border-2 border-dashed border-base-300 opacity-0 hover:opacity-50 transition-opacity"
                              aria-label="Aufgabe hinzufügen"
                            >
                              +
                            </button>
                          </li>
                        )}
                      </ul>
                    </div>
                  </td>
                );
              })}
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}
