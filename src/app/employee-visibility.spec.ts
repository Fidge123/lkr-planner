import { describe, expect, it } from "bun:test";
import type {
  EmployeeSetting,
  PlanningContactRecord,
} from "../generated/tauri";
import {
  filterVisibleEmployees,
  isPlannableEmployee,
} from "./employee-visibility";

const monteurWithCalendar: PlanningContactRecord = {
  self: "/v1/contacts/1",
  full_name: "Max Monteur",
  category: "Monteur",
};

const monteurWithoutCalendar: PlanningContactRecord = {
  self: "/v1/contacts/2",
  full_name: "Mona Ohnekalender",
  category: "Monteur",
};

const testEmployee: PlanningContactRecord = {
  self: "/v1/contacts/3",
  full_name: "Bea Test",
  category: "Test",
};

const settings: EmployeeSetting[] = [
  {
    dayliteContactReference: "/v1/contacts/1",
    zepPrimaryCalendar: "https://app.zep.de/caldav/admin/max-einsatz/",
  },
  {
    dayliteContactReference: "/v1/contacts/2",
    zepPrimaryCalendar: null,
  },
  {
    dayliteContactReference: "/v1/contacts/3",
    zepPrimaryCalendar: "https://app.zep.de/caldav/admin/bea-einsatz/",
  },
];

describe("isPlannableEmployee", () => {
  it("is true for a non-test employee with a configured calendar", () => {
    expect(isPlannableEmployee(monteurWithCalendar, settings)).toBe(true);
  });

  it("is false when no calendar is configured", () => {
    expect(isPlannableEmployee(monteurWithoutCalendar, settings)).toBe(false);
  });

  it("is false for a test employee even with a configured calendar", () => {
    expect(isPlannableEmployee(testEmployee, settings)).toBe(false);
  });

  it("treats the Test category case-insensitively", () => {
    expect(
      isPlannableEmployee({ ...testEmployee, category: "  test " }, settings),
    ).toBe(false);
  });

  it("is false when there is no matching employee setting", () => {
    expect(isPlannableEmployee(monteurWithCalendar, [])).toBe(false);
  });
});

describe("filterVisibleEmployees", () => {
  const employees = [monteurWithCalendar, monteurWithoutCalendar, testEmployee];

  it("hides non-plannable employees when the toggle is enabled", () => {
    const visible = filterVisibleEmployees(employees, settings, true);
    expect(visible).toEqual([monteurWithCalendar]);
  });

  it("shows every employee when the toggle is disabled", () => {
    const visible = filterVisibleEmployees(employees, settings, false);
    expect(visible).toEqual(employees);
  });
});
