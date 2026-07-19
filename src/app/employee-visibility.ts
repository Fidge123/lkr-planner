import type {
  EmployeeSetting,
  PlanningContactRecord,
} from "../generated/tauri";

export function isPlannableEmployee(
  employee: PlanningContactRecord,
  employeeSettings: EmployeeSetting[],
): boolean {
  const category = (employee.category ?? "").trim().toLowerCase();
  if (category === "test") {
    return false;
  }

  const setting = employeeSettings.find(
    (entry) => entry.dayliteContactReference === employee.self,
  );
  return Boolean(setting?.zepPrimaryCalendar?.trim());
}

export function filterVisibleEmployees(
  employees: PlanningContactRecord[],
  employeeSettings: EmployeeSetting[],
  hideNonPlannableEmployees: boolean,
): PlanningContactRecord[] {
  if (!hideNonPlannableEmployees) {
    return employees;
  }

  return employees.filter((employee) =>
    isPlannableEmployee(employee, employeeSettings),
  );
}
