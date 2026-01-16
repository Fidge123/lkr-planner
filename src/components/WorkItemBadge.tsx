import { employees } from "../data/dummyData";
import type { Employee, WorkItem } from "../types";

interface WorkItemBadgeProps {
  workItem: WorkItem;
  dayIndex: number;
  currentEmployeeId: string;
}

export function WorkItemBadge({
  workItem,
  dayIndex,
  currentEmployeeId,
}: WorkItemBadgeProps) {
  const sortedDays = [...workItem.days].sort((a, b) => a - b);
  const isFirstDay = sortedDays[0] === dayIndex;
  const prevDayIncluded = workItem.days.includes(dayIndex - 1);
  const nextDayIncluded = workItem.days.includes(dayIndex + 1);

  // Determine visual continuity
  const continuesFromPrev = prevDayIncluded;
  const continuesToNext = nextDayIncluded;
  const isPaused = !prevDayIncluded && !isFirstDay;
  const isResumed = isPaused;

  // Get other assignees (excluding current employee)
  const otherAssignees = workItem.assignedEmployeeIds
    .filter((id) => id !== currentEmployeeId)
    .map((id) => employees.find((e) => e.id === id))
    .filter((e): e is Employee => e !== undefined);

  const hasMultipleAssignees = otherAssignees.length > 0;

  return (
    <div
      className={`
        ${workItem.color} text-base-100
        px-2 py-1 text-xs font-medium
        flex items-center justify-between gap-1
        min-h-[28px] relative
        ${continuesFromPrev ? "rounded-l-none -ml-1 pl-3" : "rounded-l-lg"}
        ${continuesToNext ? "rounded-r-none -mr-1 pr-3" : "rounded-r-lg"}
        ${isResumed ? "border-l-2 border-dashed border-base-100/50" : ""}
        transition-all duration-200 hover:brightness-110 hover:scale-[1.02]
        cursor-pointer shadow-sm
      `}
      title={`${workItem.title} - ${workItem.project}${hasMultipleAssignees ? `\nEbenfalls zugewiesen: ${otherAssignees.map((e) => e.name).join(", ")}` : ""}`}
    >
      <span className="truncate flex-1">{workItem.title}</span>

      {/* Show indicator for multi-assignee work items */}
      {hasMultipleAssignees && (
        <div className="flex -space-x-1">
          {otherAssignees.slice(0, 2).map((assignee) => (
            <div
              key={assignee.id}
              className="w-4 h-4 rounded-full bg-base-100/30 text-[8px] flex items-center justify-center font-bold border border-base-100/20"
              title={assignee.name}
            >
              {assignee.name[0]}
            </div>
          ))}
          {otherAssignees.length > 2 && (
            <div className="w-4 h-4 rounded-full bg-base-100/30 text-[8px] flex items-center justify-center font-bold border border-base-100/20">
              +{otherAssignees.length - 2}
            </div>
          )}
        </div>
      )}

      {/* Visual indicator for continuation */}
      {continuesToNext && (
        <div className="absolute right-0 top-1/2 -translate-y-1/2 w-2 h-2 bg-inherit" />
      )}
    </div>
  );
}
