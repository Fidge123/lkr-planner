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

export const workItems: WorkItem[] = [
  {
    id: "wi-1",
    title: "API-Integration",
    project: "Kundenportal",
    color: "bg-primary",
    days: ["2026-01-26", "2026-01-27", "2026-01-28"],
    assignedEmployeeIds: ["emp-1", "emp-4"],
  },
  {
    id: "wi-2",
    title: "UI-Entwürfe",
    project: "Mobile App",
    color: "bg-secondary",
    days: ["2026-01-26", "2026-01-27"],
    assignedEmployeeIds: ["emp-3"],
  },
  {
    id: "wi-3",
    title: "Sprint-Planung",
    project: "Intern",
    color: "bg-accent",
    days: ["2026-01-26"],
    assignedEmployeeIds: ["emp-2"],
  },
  {
    id: "wi-4",
    title: "Datenbank-Migration",
    project: "Altsystem",
    color: "bg-info",
    days: ["2026-01-27", "2026-01-28", "2026-01-30"],
    assignedEmployeeIds: ["emp-4", "emp-6"],
  },
  {
    id: "wi-5",
    title: "Test-Automatisierung",
    project: "Kundenportal",
    color: "bg-success",
    days: ["2026-01-28", "2026-01-29", "2026-01-30"],
    assignedEmployeeIds: ["emp-5"],
  },
  {
    id: "wi-6",
    title: "Code-Review",
    project: "Mobile App",
    color: "bg-warning",
    days: ["2026-01-29"],
    assignedEmployeeIds: ["emp-1", "emp-7"],
  },
  {
    id: "wi-7",
    title: "CI/CD-Pipeline",
    project: "Infrastruktur",
    color: "bg-error",
    days: ["2026-01-26", "2026-01-28", "2026-01-30"],
    assignedEmployeeIds: ["emp-6"],
  },
  {
    id: "wi-8",
    title: "Nutzerforschung",
    project: "Mobile App",
    color: "bg-primary/70",
    days: ["2026-01-27", "2026-01-28"],
    assignedEmployeeIds: ["emp-3", "emp-2"],
  },
  {
    id: "wi-9",
    title: "Performance-Audit",
    project: "Kundenportal",
    color: "bg-secondary/70",
    days: ["2026-01-29", "2026-01-30"],
    assignedEmployeeIds: ["emp-7"],
  },
  {
    id: "wi-10",
    title: "Dokumentation",
    project: "Intern",
    color: "bg-accent/70",
    days: ["2026-01-30"],
    assignedEmployeeIds: ["emp-2"],
  },
  {
    id: "wi-11",
    title: "Fehlerbehebung",
    project: "Kundenportal",
    color: "bg-info/70",
    days: ["2026-01-26", "2026-01-27"],
    assignedEmployeeIds: ["emp-7"],
  },
  {
    id: "wi-12",
    title: "Sicherheitsüberprüfung",
    project: "Infrastruktur",
    color: "bg-success/70",
    days: ["2026-01-28", "2026-01-29"],
    assignedEmployeeIds: ["emp-6", "emp-4"],
  },
  {
    id: "wi-13",
    title: "Stakeholder-Meeting",
    project: "Mobile App",
    color: "bg-warning/70",
    days: ["2026-01-27"],
    assignedEmployeeIds: ["emp-2", "emp-3"],
  },
  {
    id: "wi-14",
    title: "Feature-Entwicklung",
    project: "Kundenportal",
    color: "bg-error/70",
    days: ["2026-01-29", "2026-01-30"],
    assignedEmployeeIds: ["emp-1"],
  },
  {
    id: "wi-15",
    title: "Integrationstests",
    project: "Altsystem",
    color: "bg-neutral",
    days: ["2026-01-26", "2026-01-29"],
    assignedEmployeeIds: ["emp-5"],
  },
];

// Helper function to get work items for a specific employee and day
export function getWorkItemsForCell(employeeId: string, day: Date): WorkItem[] {
  return workItems.filter(
    (wi) =>
      wi.assignedEmployeeIds.includes(employeeId) &&
      wi.days
        .map((d) => d.slice(0, 10))
        .includes(day.toISOString().slice(0, 10)),
  );
}
