import { employees, getWorkItemsForCell } from "../data/dummyData";
import type { WeekDay } from "../types";
import { WorkItemBadge } from "./WorkItemBadge";

function getWeekDays(): WeekDay[] {
  const today = new Date();
  const currentDay = today.getDay();
  // Calculate Monday of the current week (or next week if today is weekend)
  const mondayOffset =
    currentDay === 0 ? 1 : currentDay === 6 ? 2 : 1 - currentDay;
  const monday = new Date(today);
  monday.setDate(today.getDate() + mondayOffset);

  const weekDays: WeekDay[] = [];
  const dayNames = ["Montag", "Dienstag", "Mittwoch", "Donnerstag", "Freitag"];
  const shortNames = ["Mo", "Di", "Mi", "Do", "Fr"];

  for (let i = 0; i < 5; i++) {
    const date = new Date(monday);
    date.setDate(monday.getDate() + i);
    weekDays.push({
      index: i,
      name: dayNames[i],
      shortName: shortNames[i],
      date,
    });
  }

  return weekDays;
}

export function PlanningGrid() {
  const weekDays = getWeekDays();

  return (
    <div className="w-full h-full overflow-auto p-4">
      <div className="min-w-[900px]">
        {/* Header */}
        <div className="grid grid-cols-[200px_repeat(5,1fr)] gap-1 mb-2">
          {/* Empty corner cell */}
          <div className="p-3 bg-base-200 rounded-lg flex items-center justify-center">
            <span className="text-sm font-semibold text-base-content/70">
              Mitarbeiter
            </span>
          </div>

          {/* Day headers */}
          {weekDays.map((day) => (
            <div
              key={day.index}
              className="p-3 bg-base-200 rounded-lg text-center"
            >
              <div className="font-bold text-base-content">{day.shortName}</div>
              <div className="text-xs text-base-content/60">
                {day.date.toLocaleDateString("de-DE", {
                  day: "2-digit",
                  month: "2-digit",
                })}
              </div>
            </div>
          ))}
        </div>

        {/* Employee rows */}
        {employees.map((employee, empIndex) => (
          <div
            key={employee.id}
            className={`grid grid-cols-[200px_repeat(5,1fr)] gap-1 mb-1 ${
              empIndex % 2 === 0 ? "bg-base-100" : "bg-base-200/30"
            } rounded-lg`}
          >
            {/* Employee info cell */}
            <div className="p-3 flex items-center gap-3 border-r border-base-300">
              <div className="flex-1 min-w-0">
                <div className="font-semibold text-sm truncate">
                  {employee.name}
                </div>
                <div className="text-xs text-base-content/60 truncate">
                  {employee.role}
                </div>
              </div>
            </div>

            {/* Day cells */}
            {weekDays.map((day) => {
              const cellWorkItems = getWorkItemsForCell(employee.id, day.index);
              const displayItems = cellWorkItems.slice(0, 3);
              const hasOverflow = cellWorkItems.length > 3;

              return (
                <div
                  key={`${employee.id}-${day.index}`}
                  className="p-2 min-h-[100px] bg-base-100 rounded-lg border border-base-300/50 hover:border-primary/30 hover:bg-base-200/20 transition-colors"
                >
                  <div className="flex flex-col gap-1 h-full">
                    {displayItems.map((workItem) => (
                      <WorkItemBadge
                        key={workItem.id}
                        workItem={workItem}
                        dayIndex={day.index}
                        currentEmployeeId={employee.id}
                      />
                    ))}

                    {hasOverflow && (
                      <div className="text-xs text-base-content/50 text-center mt-auto">
                        +{cellWorkItems.length - 3} weitere
                      </div>
                    )}

                    {displayItems.length === 0 && (
                      <div className="flex-1 flex items-center justify-center">
                        <div className="w-8 h-8 rounded-full border-2 border-dashed border-base-300 flex items-center justify-center opacity-0 hover:opacity-50 transition-opacity cursor-pointer">
                          <span className="text-lg text-base-content/30">
                            +
                          </span>
                        </div>
                      </div>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        ))}
      </div>
    </div>
  );
}
