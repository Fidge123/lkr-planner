import type { Employee, WorkItem } from "../types";

export const employees: Employee[] = [
  {
    id: "emp-1",
    name: "Anna Schmidt",
    role: "Senior Developer",
    avatar: "AS",
  },
  {
    id: "emp-2",
    name: "Max Müller",
    role: "Project Manager",
    avatar: "MM",
  },
  {
    id: "emp-3",
    name: "Lisa Weber",
    role: "UI/UX Designer",
    avatar: "LW",
  },
  {
    id: "emp-4",
    name: "Tom Fischer",
    role: "Backend Developer",
    avatar: "TF",
  },
  {
    id: "emp-5",
    name: "Sarah Koch",
    role: "QA Engineer",
    avatar: "SK",
  },
  {
    id: "emp-6",
    name: "Jan Becker",
    role: "DevOps Engineer",
    avatar: "JB",
  },
  {
    id: "emp-7",
    name: "Maria Hofmann",
    role: "Full Stack Developer",
    avatar: "MH",
  },
];

// Work items with various patterns:
// - Multi-day spans
// - Paused and resumed (gaps in days)
// - Multiple assignees
export const workItems: WorkItem[] = [
  {
    id: "wi-1",
    title: "API Integration",
    project: "Customer Portal",
    color: "bg-primary",
    days: [0, 1, 2], // Mon-Wed continuous
    assignedEmployeeIds: ["emp-1", "emp-4"],
  },
  {
    id: "wi-2",
    title: "UI Mockups",
    project: "Mobile App",
    color: "bg-secondary",
    days: [0, 1], // Mon-Tue
    assignedEmployeeIds: ["emp-3"],
  },
  {
    id: "wi-3",
    title: "Sprint Planning",
    project: "Internal",
    color: "bg-accent",
    days: [0], // Monday only
    assignedEmployeeIds: ["emp-2"],
  },
  {
    id: "wi-4",
    title: "Database Migration",
    project: "Legacy System",
    color: "bg-info",
    days: [1, 2, 4], // Tue-Wed, then resumed Friday (paused Thursday)
    assignedEmployeeIds: ["emp-4", "emp-6"],
  },
  {
    id: "wi-5",
    title: "Test Automation",
    project: "Customer Portal",
    color: "bg-success",
    days: [2, 3, 4], // Wed-Fri
    assignedEmployeeIds: ["emp-5"],
  },
  {
    id: "wi-6",
    title: "Code Review",
    project: "Mobile App",
    color: "bg-warning",
    days: [3], // Thursday only
    assignedEmployeeIds: ["emp-1", "emp-7"],
  },
  {
    id: "wi-7",
    title: "CI/CD Pipeline",
    project: "Infrastructure",
    color: "bg-error",
    days: [0, 2, 4], // Mon, Wed, Fri (intermittent)
    assignedEmployeeIds: ["emp-6"],
  },
  {
    id: "wi-8",
    title: "User Research",
    project: "Mobile App",
    color: "bg-primary/70",
    days: [1, 2], // Tue-Wed
    assignedEmployeeIds: ["emp-3", "emp-2"],
  },
  {
    id: "wi-9",
    title: "Performance Audit",
    project: "Customer Portal",
    color: "bg-secondary/70",
    days: [3, 4], // Thu-Fri
    assignedEmployeeIds: ["emp-7"],
  },
  {
    id: "wi-10",
    title: "Documentation",
    project: "Internal",
    color: "bg-accent/70",
    days: [4], // Friday only
    assignedEmployeeIds: ["emp-2"],
  },
  {
    id: "wi-11",
    title: "Bug Fixes",
    project: "Customer Portal",
    color: "bg-info/70",
    days: [0, 1], // Mon-Tue
    assignedEmployeeIds: ["emp-7"],
  },
  {
    id: "wi-12",
    title: "Security Review",
    project: "Infrastructure",
    color: "bg-success/70",
    days: [2, 3], // Wed-Thu
    assignedEmployeeIds: ["emp-6", "emp-4"],
  },
  {
    id: "wi-13",
    title: "Stakeholder Meeting",
    project: "Mobile App",
    color: "bg-warning/70",
    days: [1], // Tuesday only
    assignedEmployeeIds: ["emp-2", "emp-3"],
  },
  {
    id: "wi-14",
    title: "Feature Development",
    project: "Customer Portal",
    color: "bg-error/70",
    days: [3, 4], // Thu-Fri
    assignedEmployeeIds: ["emp-1"],
  },
  {
    id: "wi-15",
    title: "Integration Testing",
    project: "Legacy System",
    color: "bg-neutral",
    days: [0, 3], // Mon, Thu (gap for pause)
    assignedEmployeeIds: ["emp-5"],
  },
];

// Helper function to get work items for a specific employee and day
export function getWorkItemsForCell(
  employeeId: string,
  dayIndex: number,
): WorkItem[] {
  return workItems.filter(
    (wi) =>
      wi.assignedEmployeeIds.includes(employeeId) && wi.days.includes(dayIndex),
  );
}
