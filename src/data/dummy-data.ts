import type {
  Assignment,
  AssignmentSource,
  AssignmentSyncStatus,
  Employee,
  Project,
} from "../domain/planning";

export interface PlanningCellProject {
  id: string;
  title: string;
  color: string;
}

interface AssignmentTemplate {
  id: string;
  projectId: string;
  days: string[];
  employeeIds: string[];
  source: AssignmentSource;
  syncStatus: AssignmentSyncStatus;
}

const projectStatusClasses: Record<string, string> = {
  new: "bg-primary",
  in_progress: "bg-secondary",
  done: "bg-success",
  archived: "bg-neutral",
  unknown: "bg-base-300",
};

export const employees: Employee[] = [
  {
    id: "emp-1",
    dayliteReference: "/v1/contacts/1001",
    name: "Anna Schmidt",
    skills: ["Backend", "API"],
    homeLocation: "Köln, Deutschland",
    primaryIcalUrl: "https://calendar.example.com/anna/primary.ics",
    absenceIcalUrl: "https://calendar.example.com/anna/absence.ics",
    active: true,
  },
  {
    id: "emp-2",
    dayliteReference: "/v1/contacts/1002",
    name: "Max Müller",
    skills: ["Projektleitung", "Koordination"],
    homeLocation: "Bonn, Deutschland",
    primaryIcalUrl: "https://calendar.example.com/max/primary.ics",
    absenceIcalUrl: "https://calendar.example.com/max/absence.ics",
    active: true,
  },
  {
    id: "emp-3",
    dayliteReference: "/v1/contacts/1003",
    name: "Lisa Weber",
    skills: ["UI/UX", "Research"],
    homeLocation: "Köln, Deutschland",
    primaryIcalUrl: "https://calendar.example.com/lisa/primary.ics",
    absenceIcalUrl: "https://calendar.example.com/lisa/absence.ics",
    active: true,
  },
  {
    id: "emp-4",
    dayliteReference: "/v1/contacts/1004",
    name: "Tom Fischer",
    skills: ["Backend", "Datenbank"],
    homeLocation: "Leverkusen, Deutschland",
    primaryIcalUrl: "https://calendar.example.com/tom/primary.ics",
    absenceIcalUrl: "https://calendar.example.com/tom/absence.ics",
    active: true,
  },
  {
    id: "emp-5",
    dayliteReference: "/v1/contacts/1005",
    name: "Sarah Koch",
    skills: ["QA", "Testautomatisierung"],
    homeLocation: "Düsseldorf, Deutschland",
    primaryIcalUrl: "https://calendar.example.com/sarah/primary.ics",
    absenceIcalUrl: "https://calendar.example.com/sarah/absence.ics",
    active: true,
  },
  {
    id: "emp-6",
    dayliteReference: "/v1/contacts/1006",
    name: "Jan Becker",
    skills: ["DevOps", "Security"],
    homeLocation: "Köln, Deutschland",
    primaryIcalUrl: "https://calendar.example.com/jan/primary.ics",
    absenceIcalUrl: "https://calendar.example.com/jan/absence.ics",
    active: true,
  },
  {
    id: "emp-7",
    dayliteReference: "/v1/contacts/1007",
    name: "Maria Hofmann",
    skills: ["Fullstack", "Performance"],
    homeLocation: "Siegburg, Deutschland",
    primaryIcalUrl: "https://calendar.example.com/maria/primary.ics",
    absenceIcalUrl: "https://calendar.example.com/maria/absence.ics",
    active: true,
  },
];

export const projects: Project[] = [
  {
    id: "proj-1",
    dayliteReference: "/v1/projects/3001",
    name: "Kundenportal",
    status: "in_progress",
  },
  {
    id: "proj-2",
    dayliteReference: "/v1/projects/3002",
    name: "Mobile App",
    status: "new",
  },
  {
    id: "proj-3",
    dayliteReference: "/v1/projects/3003",
    name: "Intern",
    status: "done",
  },
  {
    id: "proj-4",
    dayliteReference: "/v1/projects/3004",
    name: "Altsystem",
    status: "in_progress",
  },
  {
    id: "proj-5",
    dayliteReference: "/v1/projects/3005",
    name: "Infrastruktur",
    status: "new",
  },
];

const assignmentTemplates: AssignmentTemplate[] = [
  {
    id: "asg-1",
    projectId: "proj-1",
    days: ["2026-01-26", "2026-01-27", "2026-01-28"],
    employeeIds: ["emp-1", "emp-4"],
    source: "manual",
    syncStatus: "synced",
  },
  {
    id: "asg-2",
    projectId: "proj-2",
    days: ["2026-01-26", "2026-01-27"],
    employeeIds: ["emp-3"],
    source: "manual",
    syncStatus: "synced",
  },
  {
    id: "asg-3",
    projectId: "proj-3",
    days: ["2026-01-26"],
    employeeIds: ["emp-2"],
    source: "manual",
    syncStatus: "pending",
  },
  {
    id: "asg-4",
    projectId: "proj-4",
    days: ["2026-01-27", "2026-01-28", "2026-01-30"],
    employeeIds: ["emp-4", "emp-6"],
    source: "manual",
    syncStatus: "synced",
  },
  {
    id: "asg-5",
    projectId: "proj-1",
    days: ["2026-01-28", "2026-01-29", "2026-01-30"],
    employeeIds: ["emp-5"],
    source: "manual",
    syncStatus: "pending",
  },
  {
    id: "asg-6",
    projectId: "proj-2",
    days: ["2026-01-29"],
    employeeIds: ["emp-1", "emp-7"],
    source: "manual",
    syncStatus: "pending",
  },
  {
    id: "asg-7",
    projectId: "proj-5",
    days: ["2026-01-26", "2026-01-28", "2026-01-30"],
    employeeIds: ["emp-6"],
    source: "manual",
    syncStatus: "synced",
  },
  {
    id: "asg-8",
    projectId: "proj-2",
    days: ["2026-01-27", "2026-01-28"],
    employeeIds: ["emp-3", "emp-2"],
    source: "manual",
    syncStatus: "synced",
  },
  {
    id: "asg-9",
    projectId: "proj-1",
    days: ["2026-01-29", "2026-01-30"],
    employeeIds: ["emp-7"],
    source: "manual",
    syncStatus: "pending",
  },
  {
    id: "asg-10",
    projectId: "proj-3",
    days: ["2026-01-30"],
    employeeIds: ["emp-2"],
    source: "manual",
    syncStatus: "synced",
  },
  {
    id: "asg-11",
    projectId: "proj-1",
    days: ["2026-01-26", "2026-01-27"],
    employeeIds: ["emp-7"],
    source: "manual",
    syncStatus: "synced",
  },
  {
    id: "asg-12",
    projectId: "proj-5",
    days: ["2026-01-28", "2026-01-29"],
    employeeIds: ["emp-6", "emp-4"],
    source: "manual",
    syncStatus: "synced",
  },
  {
    id: "asg-13",
    projectId: "proj-2",
    days: ["2026-01-27"],
    employeeIds: ["emp-2", "emp-3"],
    source: "manual",
    syncStatus: "synced",
  },
  {
    id: "asg-14",
    projectId: "proj-1",
    days: ["2026-01-29", "2026-01-30"],
    employeeIds: ["emp-1"],
    source: "manual",
    syncStatus: "pending",
  },
  {
    id: "asg-15",
    projectId: "proj-4",
    days: ["2026-01-26", "2026-01-29"],
    employeeIds: ["emp-5"],
    source: "manual",
    syncStatus: "synced",
  },
];

export const assignments: Assignment[] = assignmentTemplates.flatMap(
  (template) =>
    template.employeeIds.flatMap((employeeId) =>
      template.days.map((day) => ({
        id: `${template.id}-${employeeId}-${day}`,
        employeeId,
        projectId: template.projectId,
        period: {
          startDate: day,
          endDate: day,
        },
        source: template.source,
        syncStatus: template.syncStatus,
      })),
    ),
);

const projectsById = new Map(projects.map((project) => [project.id, project]));

export function getWorkItemsForCell(
  employeeId: string,
  day: Date,
): PlanningCellProject[] {
  const isoDay = day.toISOString().slice(0, 10);

  return assignments
    .filter(
      (assignment) =>
        assignment.employeeId === employeeId &&
        isDayInAssignmentPeriod(isoDay, assignment),
    )
    .map((assignment) => {
      const project = projectsById.get(assignment.projectId);
      const status = project?.status ?? "unknown";

      return {
        id: assignment.id,
        title: project?.name ?? assignment.projectId,
        color: projectStatusClasses[status] ?? projectStatusClasses.unknown,
      };
    });
}

function isDayInAssignmentPeriod(
  isoDay: string,
  assignment: Assignment,
): boolean {
  return (
    isoDay >= assignment.period.startDate && isoDay <= assignment.period.endDate
  );
}
