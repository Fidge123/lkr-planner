import { employees } from "../data/dummy-data";
import { TimetableHeader } from "./components/timetable-header";
import { TimetableRow } from "./components/timetable-row";
import { getWeekDays } from "./util";

export function PlanningGrid({ weekOffset }: Props) {
  const weekDays = getWeekDays(weekOffset);

  return (
    <section className="w-full h-full overflow-auto">
      <table className="table table-zebra table-fixed border-collapse">
        <thead className="text-base-content">
          <tr className="divide-x divide-slate-300 bg-base-200">
            <th className="w-36 p-4 font-bold">Mitarbeiter</th>
            {weekDays.map((day, i) => (
              <TimetableHeader key={day.getTime()} day={day} />
            ))}
          </tr>
        </thead>
        <tbody>
          {employees.map((employee) => (
            <TimetableRow
              key={employee.id}
              employee={employee}
              weekDays={weekDays}
            />
          ))}
        </tbody>
      </table>
    </section>
  );
}

interface Props {
  weekOffset: number;
}
