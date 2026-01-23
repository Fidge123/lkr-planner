import { employees, getWorkItemsForCell } from "../data/dummyData";

interface PlanningGridProps {
  weekOffset: number;
}

function getWeekDays(weekOffset: number) {
  const today = new Date();
  const currentDay = today.getDay();
  const dayInMs = 1000 * 60 * 60 * 24;
  // Calculate Monday of the current week (or next week if today is weekend)
  const mondayOffset =
    currentDay === 0 ? 1 : currentDay === 6 ? 2 : 1 - currentDay;
  const monday = new Date(today);
  monday.setDate(today.getDate() + mondayOffset + weekOffset * 7);

  return [
    monday,
    new Date(monday.getTime() + dayInMs),
    new Date(monday.getTime() + dayInMs * 2),
    new Date(monday.getTime() + dayInMs * 3),
    new Date(monday.getTime() + dayInMs * 4),
  ];
}

export function PlanningGrid({ weekOffset }: PlanningGridProps) {
  const weekDays = getWeekDays(weekOffset);
  const todayIndex = weekOffset === 0 ? new Date().getDay() - 1 : undefined;

  return (
    <section className="w-full h-full overflow-auto">
      <table className="table table-zebra table-fixed border-collapse">
        <thead className="text-base-content">
          <tr className="divide-x divide-slate-300 bg-base-200">
            <th className="w-36 p-4 font-bold">Mitarbeiter</th>
            {weekDays.map((day, i) => (
              <th
                key={day.getTime()}
                className={`text-center ${
                  i === todayIndex ? "bg-primary text-primary-content" : ""
                }`}
              >
                <time dateTime={day.toISOString()}>
                  {day.toLocaleDateString("de-DE", {
                    weekday: "long",
                    day: "2-digit",
                    month: "2-digit",
                  })}
                </time>
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {employees.map((employee) => (
            <tr key={employee.id} className="divide-x divide-slate-300">
              <th className="p-4 align-top font-normal">{employee.name}</th>

              {weekDays.map((day, i) => {
                const cellWorkItems = getWorkItemsForCell(employee.id, day);

                return (
                  <td
                    key={`${employee.id}-${day.toISOString()}`}
                    className={`align-top p-2 ${i === todayIndex ? "bg-primary/10" : ""}`}
                  >
                    <ul className="flex flex-col gap-1 list-none">
                      {cellWorkItems.map((workItem) => (
                        <li key={workItem.id}>
                          <button
                            type="button"
                            className={`btn btn-block ${workItem.color} text-base-100 p-2 rounded-lg`}
                          >
                            <h4 className="truncate flex-1 font-medium">
                              {workItem.title}
                            </h4>
                          </button>
                        </li>
                      ))}
                      <li>
                        <button
                          type="button"
                          className="btn btn-dash btn-block rounded-lg opacity-20 hover:opacity-80 transition-opacity"
                          aria-label="Aufgabe hinzufügen"
                        >
                          +
                        </button>
                      </li>
                    </ul>
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
