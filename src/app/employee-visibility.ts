import type {
  EmployeeSetting,
  PlanningContactRecord,
} from "../generated/tauri";

/**
 * An employee is "plannable" when they can be scheduled in the planning view:
 * they are not a test contact (Daylite category "Test") and have a primary
 * calendar configured. Daylite is the source of truth for the calendar, mirrored
 * into the employee settings.
 */
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

/**
 * Returns the employees to render in the planning view. When the
 * "hide non-plannable employees" toggle is enabled, employees without a
 * configured calendar and test employees are removed; otherwise every fetched
 * employee is shown.
 */
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
