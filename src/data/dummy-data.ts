import type {
  Assignment,
  AssignmentSource,
  AssignmentSyncStatus,
  DayliteContactRecord,
  DayliteProjectRecord,
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

export const employees: DayliteContactRecord[] = [
  {
    self: "/v1/contacts/1001",
    full_name: "Anna Schmidt",
    keywords: ["Backend", "API"],
    addresses: [
      {
        city: "Köln",
        country: "Deutschland",
      },
    ],
    urls: [
      {
        label: "Einsatz iCal",
        url: "https://calendar.example.com/anna/primary.ics",
      },
      {
        label: "Abwesenheit iCal",
        url: "https://calendar.example.com/anna/absence.ics",
      },
    ],
  },
  {
    self: "/v1/contacts/1002",
    full_name: "Max Müller",
    keywords: ["Projektleitung", "Koordination"],
    addresses: [
      {
        city: "Bonn",
        country: "Deutschland",
      },
    ],
    urls: [
      {
        label: "Einsatz iCal",
        url: "https://calendar.example.com/max/primary.ics",
      },
      {
        label: "Abwesenheit iCal",
        url: "https://calendar.example.com/max/absence.ics",
      },
    ],
  },
  {
    self: "/v1/contacts/1003",
    full_name: "Lisa Weber",
    keywords: ["UI/UX", "Research"],
    addresses: [
      {
        city: "Köln",
        country: "Deutschland",
      },
    ],
    urls: [
      {
        label: "Einsatz iCal",
        url: "https://calendar.example.com/lisa/primary.ics",
      },
      {
        label: "Abwesenheit iCal",
        url: "https://calendar.example.com/lisa/absence.ics",
      },
    ],
  },
  {
    self: "/v1/contacts/1004",
    full_name: "Tom Fischer",
    nickname: "Tom",
    keywords: ["Backend", "Datenbank"],
    addresses: [
      {
        city: "Leverkusen",
        country: "Deutschland",
      },
    ],
    urls: [
      {
        label: "Einsatz iCal",
        url: "https://calendar.example.com/tom/primary.ics",
      },
      {
        label: "Abwesenheit iCal",
        url: "https://calendar.example.com/tom/absence.ics",
      },
    ],
  },
  {
    self: "/v1/contacts/1005",
    full_name: "Sarah Koch",
    keywords: ["QA", "Testautomatisierung"],
    addresses: [
      {
        city: "Düsseldorf",
        country: "Deutschland",
      },
    ],
    urls: [
      {
        label: "Einsatz iCal",
        url: "https://calendar.example.com/sarah/primary.ics",
      },
      {
        label: "Abwesenheit iCal",
        url: "https://calendar.example.com/sarah/absence.ics",
      },
    ],
  },
  {
    self: "/v1/contacts/1006",
    full_name: "Jan Becker",
    keywords: ["DevOps", "Security"],
    addresses: [
      {
        city: "Köln",
        country: "Deutschland",
      },
    ],
    urls: [
      {
        label: "Einsatz iCal",
        url: "https://calendar.example.com/jan/primary.ics",
      },
      {
        label: "Abwesenheit iCal",
        url: "https://calendar.example.com/jan/absence.ics",
      },
    ],
  },
  {
    self: "/v1/contacts/1007",
    full_name: "Maria Hofmann",
    keywords: ["Fullstack", "Performance"],
    addresses: [
      {
        city: "Siegburg",
        country: "Deutschland",
      },
    ],
    urls: [
      {
        label: "Einsatz iCal",
        url: "https://calendar.example.com/maria/primary.ics",
      },
      {
        label: "Abwesenheit iCal",
        url: "https://calendar.example.com/maria/absence.ics",
      },
    ],
  },
];

export const projects: DayliteProjectRecord[] = [
  {
    self: "/v1/projects/3001",
    name: "Kundenportal",
    status: "in_progress",
  },
  {
    self: "/v1/projects/3002",
    name: "Mobile App",
    status: "new",
  },
  {
    self: "/v1/projects/3003",
    name: "Intern",
    status: "done",
  },
  {
    self: "/v1/projects/3004",
    name: "Altsystem",
    status: "in_progress",
  },
  {
    self: "/v1/projects/3005",
    name: "Infrastruktur",
    status: "new",
  },
];

const assignmentTemplates: AssignmentTemplate[] = [
  {
    id: "asg-1",
    projectId: "/v1/projects/3001",
    days: ["2026-01-26", "2026-01-27", "2026-01-28"],
    employeeIds: ["/v1/contacts/1001", "/v1/contacts/1004"],
    source: "app",
    syncStatus: "synced",
  },
  {
    id: "asg-2",
    projectId: "/v1/projects/3002",
    days: ["2026-01-26", "2026-01-27"],
    employeeIds: ["/v1/contacts/1003"],
    source: "app",
    syncStatus: "synced",
  },
  {
    id: "asg-3",
    projectId: "/v1/projects/3003",
    days: ["2026-01-26"],
    employeeIds: ["/v1/contacts/1002"],
    source: "app",
    syncStatus: "pending",
  },
  {
    id: "asg-4",
    projectId: "/v1/projects/3004",
    days: ["2026-01-27", "2026-01-28", "2026-01-30"],
    employeeIds: ["/v1/contacts/1004", "/v1/contacts/1006"],
    source: "app",
    syncStatus: "synced",
  },
  {
    id: "asg-5",
    projectId: "/v1/projects/3001",
    days: ["2026-01-28", "2026-01-29", "2026-01-30"],
    employeeIds: ["/v1/contacts/1005"],
    source: "app",
    syncStatus: "pending",
  },
  {
    id: "asg-6",
    projectId: "/v1/projects/3002",
    days: ["2026-01-29"],
    employeeIds: ["/v1/contacts/1001", "/v1/contacts/1007"],
    source: "app",
    syncStatus: "pending",
  },
  {
    id: "asg-7",
    projectId: "/v1/projects/3005",
    days: ["2026-01-26", "2026-01-28", "2026-01-30"],
    employeeIds: ["/v1/contacts/1006"],
    source: "app",
    syncStatus: "synced",
  },
  {
    id: "asg-8",
    projectId: "/v1/projects/3002",
    days: ["2026-01-27", "2026-01-28"],
    employeeIds: ["/v1/contacts/1003", "/v1/contacts/1002"],
    source: "app",
    syncStatus: "synced",
  },
  {
    id: "asg-9",
    projectId: "/v1/projects/3001",
    days: ["2026-01-29", "2026-01-30"],
    employeeIds: ["/v1/contacts/1007"],
    source: "app",
    syncStatus: "pending",
  },
  {
    id: "asg-10",
    projectId: "/v1/projects/3003",
    days: ["2026-01-30"],
    employeeIds: ["/v1/contacts/1002"],
    source: "app",
    syncStatus: "synced",
  },
  {
    id: "asg-11",
    projectId: "/v1/projects/3001",
    days: ["2026-01-26", "2026-01-27"],
    employeeIds: ["/v1/contacts/1007"],
    source: "app",
    syncStatus: "synced",
  },
  {
    id: "asg-12",
    projectId: "/v1/projects/3005",
    days: ["2026-01-28", "2026-01-29"],
    employeeIds: ["/v1/contacts/1006", "/v1/contacts/1004"],
    source: "app",
    syncStatus: "synced",
  },
  {
    id: "asg-13",
    projectId: "/v1/projects/3002",
    days: ["2026-01-27"],
    employeeIds: ["/v1/contacts/1002", "/v1/contacts/1003"],
    source: "app",
    syncStatus: "synced",
  },
  {
    id: "asg-14",
    projectId: "/v1/projects/3001",
    days: ["2026-01-29", "2026-01-30"],
    employeeIds: ["/v1/contacts/1001"],
    source: "app",
    syncStatus: "pending",
  },
  {
    id: "asg-15",
    projectId: "/v1/projects/3004",
    days: ["2026-01-26", "2026-01-29"],
    employeeIds: ["/v1/contacts/1005"],
    source: "app",
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

const projectsByReference = new Map(
  projects.map((project) => [project.self, project]),
);

export function getWorkItemsForCell(
  employeeReference: string,
  day: Date,
): PlanningCellProject[] {
  const isoDay = day.toISOString().slice(0, 10);

  return assignments
    .filter(
      (assignment) =>
        assignment.employeeId === employeeReference &&
        isDayInAssignmentPeriod(isoDay, assignment),
    )
    .map((assignment) => {
      const project = projectsByReference.get(assignment.projectId);
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
