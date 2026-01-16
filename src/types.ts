export interface Employee {
  id: string;
  name: string;
  role: string;
  avatar: string;
}

export interface WorkItem {
  id: string;
  title: string;
  project: string;
  color: string;
  // Days this work item spans (0 = Monday, 4 = Friday)
  days: number[];
  // Employees assigned to this work item
  assignedEmployeeIds: string[];
}

export interface DayAssignment {
  employeeId: string;
  day: number;
  workItemIds: string[];
}

export type WeekDay = {
  index: number;
  name: string;
  shortName: string;
  date: Date;
};
