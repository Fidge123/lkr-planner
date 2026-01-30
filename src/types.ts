export interface Employee {
  id: string;
  name: string;
  role: string;
}

export interface WorkItem {
  id: string;
  title: string;
  project: string;
  color: string;
  days: string[];
  assignedEmployeeIds: string[];
}

export interface DayAssignment {
  employeeId: string;
  day: number;
  workItemIds: string[];
}
