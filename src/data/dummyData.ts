import type { Employee, WorkItem } from "../types";

export const employees: Employee[] = [
  {
    id: "emp-1",
    name: "Anna Schmidt",
    role: "Senior Entwicklerin",
  },
  {
    id: "emp-2",
    name: "Max Müller",
    role: "Projektleiter",
  },
  {
    id: "emp-3",
    name: "Lisa Weber",
    role: "UI/UX Designerin",
  },
  {
    id: "emp-4",
    name: "Tom Fischer",
    role: "Backend Entwickler",
  },
  {
    id: "emp-5",
    name: "Sarah Koch",
    role: "QA Ingenieurin",
  },
  {
    id: "emp-6",
    name: "Jan Becker",
    role: "DevOps Ingenieur",
  },
  {
    id: "emp-7",
    name: "Maria Hofmann",
    role: "Fullstack Entwicklerin",
  },
];

// Work items with various patterns:
// - Multi-day spans
// - Paused and resumed (gaps in days)
// - Multiple assignees
export const workItems: WorkItem[] = [
  {
    id: "wi-1",
    title: "API-Integration",
    project: "Kundenportal",
    color: "bg-primary",
    days: [0, 1, 2], // Mo-Mi durchgehend
    assignedEmployeeIds: ["emp-1", "emp-4"],
  },
  {
    id: "wi-2",
    title: "UI-Entwürfe",
    project: "Mobile App",
    color: "bg-secondary",
    days: [0, 1], // Mo-Di
    assignedEmployeeIds: ["emp-3"],
  },
  {
    id: "wi-3",
    title: "Sprint-Planung",
    project: "Intern",
    color: "bg-accent",
    days: [0], // Nur Montag
    assignedEmployeeIds: ["emp-2"],
  },
  {
    id: "wi-4",
    title: "Datenbank-Migration",
    project: "Altsystem",
    color: "bg-info",
    days: [1, 2, 4], // Di-Mi, dann Freitag fortgesetzt (Do pausiert)
    assignedEmployeeIds: ["emp-4", "emp-6"],
  },
  {
    id: "wi-5",
    title: "Test-Automatisierung",
    project: "Kundenportal",
    color: "bg-success",
    days: [2, 3, 4], // Mi-Fr
    assignedEmployeeIds: ["emp-5"],
  },
  {
    id: "wi-6",
    title: "Code-Review",
    project: "Mobile App",
    color: "bg-warning",
    days: [3], // Nur Donnerstag
    assignedEmployeeIds: ["emp-1", "emp-7"],
  },
  {
    id: "wi-7",
    title: "CI/CD-Pipeline",
    project: "Infrastruktur",
    color: "bg-error",
    days: [0, 2, 4], // Mo, Mi, Fr (intermittierend)
    assignedEmployeeIds: ["emp-6"],
  },
  {
    id: "wi-8",
    title: "Nutzerforschung",
    project: "Mobile App",
    color: "bg-primary/70",
    days: [1, 2], // Di-Mi
    assignedEmployeeIds: ["emp-3", "emp-2"],
  },
  {
    id: "wi-9",
    title: "Performance-Audit",
    project: "Kundenportal",
    color: "bg-secondary/70",
    days: [3, 4], // Do-Fr
    assignedEmployeeIds: ["emp-7"],
  },
  {
    id: "wi-10",
    title: "Dokumentation",
    project: "Intern",
    color: "bg-accent/70",
    days: [4], // Nur Freitag
    assignedEmployeeIds: ["emp-2"],
  },
  {
    id: "wi-11",
    title: "Fehlerbehebung",
    project: "Kundenportal",
    color: "bg-info/70",
    days: [0, 1], // Mo-Di
    assignedEmployeeIds: ["emp-7"],
  },
  {
    id: "wi-12",
    title: "Sicherheitsüberprüfung",
    project: "Infrastruktur",
    color: "bg-success/70",
    days: [2, 3], // Mi-Do
    assignedEmployeeIds: ["emp-6", "emp-4"],
  },
  {
    id: "wi-13",
    title: "Stakeholder-Meeting",
    project: "Mobile App",
    color: "bg-warning/70",
    days: [1], // Nur Dienstag
    assignedEmployeeIds: ["emp-2", "emp-3"],
  },
  {
    id: "wi-14",
    title: "Feature-Entwicklung",
    project: "Kundenportal",
    color: "bg-error/70",
    days: [3, 4], // Do-Fr
    assignedEmployeeIds: ["emp-1"],
  },
  {
    id: "wi-15",
    title: "Integrationstests",
    project: "Altsystem",
    color: "bg-neutral",
    days: [0, 3], // Mo, Do (Pause dazwischen)
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
