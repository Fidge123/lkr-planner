const assignmentSources = ["manual", "daylite", "planradar", "ical"] as const;
const assignmentSyncStatuses = ["pending", "synced", "failed"] as const;
const syncSources = ["daylite", "planradar", "ical", "manual"] as const;

const primaryIcalLabels = ["einsatz", "zuweisung", "assignment", "primary"];
const absenceIcalLabels = [
  "abwesenheit",
  "absence",
  "vacation",
  "urlaub",
  "sick",
  "krank",
];

export type AssignmentSource = (typeof assignmentSources)[number];
export type AssignmentSyncStatus = (typeof assignmentSyncStatuses)[number];
export type SyncSource = (typeof syncSources)[number];

export interface Project {
  id: string;
  dayliteReference: string;
  name: string;
  status: string;
}

export interface Employee {
  id: string;
  dayliteReference: string;
  name: string;
  skills: string[];
  homeLocation: string;
  primaryIcalUrl: string;
  absenceIcalUrl: string;
  active: boolean;
}

export interface AssignmentPeriod {
  startDate: string;
  endDate: string;
}

export interface Assignment {
  id: string;
  employeeId: string;
  projectId: string;
  period: AssignmentPeriod;
  source: AssignmentSource;
  syncStatus: AssignmentSyncStatus;
}

export interface SyncIssue {
  source: SyncSource;
  code: string;
  message: string;
  timestamp: string;
}

export interface DayliteProjectRecord {
  self: string;
  name: string;
  status?: string;
  category?: string;
  keywords?: string[];
  due?: string;
  started?: string;
  create_date?: string;
  modify_date?: string;
}

export interface DayliteUrl {
  label?: string;
  url?: string;
  note?: string;
}

export interface DayliteAddress {
  label?: string;
  street?: string;
  city?: string;
  state?: string;
  zip?: string;
  postal_code?: string;
  country?: string;
  note?: string;
}

type DayliteExtraFieldValue = {
  value?: string;
};

type DayliteExtraFields = Record<string, DayliteExtraFieldValue>;

export interface DayliteContactRecord {
  self: string;
  first_name?: string;
  middle_name?: string;
  last_name?: string;
  category?: string;
  keywords?: string[];
  urls?: DayliteUrl[];
  addresses?: DayliteAddress[];
  extra_fields?: string | DayliteExtraFields;
}

export function isDayliteProjectRecord(
  value: unknown,
): value is DayliteProjectRecord {
  if (!isObject(value)) {
    return false;
  }

  if (typeof value.self !== "string" || typeof value.name !== "string") {
    return false;
  }

  if ("status" in value && value.status !== undefined) {
    return typeof value.status === "string";
  }

  return true;
}

export function isDayliteContactRecord(
  value: unknown,
): value is DayliteContactRecord {
  if (!isObject(value) || typeof value.self !== "string") {
    return false;
  }

  if ("first_name" in value && value.first_name !== undefined) {
    if (typeof value.first_name !== "string") {
      return false;
    }
  }

  if ("middle_name" in value && value.middle_name !== undefined) {
    if (typeof value.middle_name !== "string") {
      return false;
    }
  }

  if ("last_name" in value && value.last_name !== undefined) {
    if (typeof value.last_name !== "string") {
      return false;
    }
  }

  return true;
}

export function isAssignment(value: unknown): value is Assignment {
  if (!isObject(value)) {
    return false;
  }

  if (
    typeof value.id !== "string" ||
    typeof value.employeeId !== "string" ||
    typeof value.projectId !== "string"
  ) {
    return false;
  }

  if (!isObject(value.period)) {
    return false;
  }

  if (
    typeof value.period.startDate !== "string" ||
    typeof value.period.endDate !== "string"
  ) {
    return false;
  }

  if (
    typeof value.source !== "string" ||
    !assignmentSources.includes(value.source as AssignmentSource)
  ) {
    return false;
  }

  if (
    typeof value.syncStatus !== "string" ||
    !assignmentSyncStatuses.includes(value.syncStatus as AssignmentSyncStatus)
  ) {
    return false;
  }

  return true;
}

export function isSyncIssue(value: unknown): value is SyncIssue {
  if (!isObject(value)) {
    return false;
  }

  if (
    typeof value.source !== "string" ||
    !syncSources.includes(value.source as SyncSource)
  ) {
    return false;
  }

  return (
    typeof value.code === "string" &&
    typeof value.message === "string" &&
    typeof value.timestamp === "string"
  );
}

export function mapDayliteProjectRecordToProject(
  record: DayliteProjectRecord,
): Project {
  return {
    id: extractDayliteObjectId(record.self),
    dayliteReference: record.self,
    name: record.name,
    status: record.status ?? "unknown",
  };
}

export function mapDayliteContactRecordToEmployee(
  record: DayliteContactRecord,
): Employee {
  const extraFields = parseExtraFields(record.extra_fields);
  const primaryIcalUrl =
    findIcalUrlFromUrls(record.urls, primaryIcalLabels) ??
    findIcalUrlFromExtraFields(extraFields, primaryIcalLabels) ??
    "";
  const absenceIcalUrl =
    findIcalUrlFromUrls(record.urls, absenceIcalLabels) ??
    findIcalUrlFromExtraFields(extraFields, absenceIcalLabels) ??
    "";

  return {
    id: extractDayliteObjectId(record.self),
    dayliteReference: record.self,
    name: mapContactName(record),
    skills: record.keywords ?? [],
    homeLocation: mapHomeLocation(record.addresses),
    primaryIcalUrl,
    absenceIcalUrl,
    active: true,
  };
}

function mapContactName(record: DayliteContactRecord): string {
  const nameParts = [record.first_name, record.middle_name, record.last_name]
    .map((namePart) => (typeof namePart === "string" ? namePart.trim() : ""))
    .filter((namePart) => namePart.length > 0);

  return nameParts.length > 0 ? nameParts.join(" ") : record.self;
}

function mapHomeLocation(addresses: DayliteAddress[] | undefined): string {
  const address = addresses?.[0];
  if (!address) {
    return "Unbekannt";
  }

  const locationParts = [address.city, address.state, address.country]
    .map((part) => (typeof part === "string" ? part.trim() : ""))
    .filter((part) => part.length > 0);

  return locationParts.length > 0 ? locationParts.join(", ") : "Unbekannt";
}

function parseExtraFields(
  extraFields: string | DayliteExtraFields | undefined,
): DayliteExtraFields {
  if (!extraFields) {
    return {};
  }

  if (typeof extraFields === "string") {
    try {
      const parsed = JSON.parse(extraFields) as unknown;
      if (isDayliteExtraFields(parsed)) {
        return parsed;
      }
      return {};
    } catch {
      return {};
    }
  }

  return extraFields;
}

function isDayliteExtraFields(value: unknown): value is DayliteExtraFields {
  if (!isObject(value)) {
    return false;
  }

  for (const fieldValue of Object.values(value)) {
    if (!isObject(fieldValue)) {
      return false;
    }

    if ("value" in fieldValue && fieldValue.value !== undefined) {
      if (typeof fieldValue.value !== "string") {
        return false;
      }
    }
  }

  return true;
}

function findIcalUrlFromUrls(
  urls: DayliteUrl[] | undefined,
  labelMatchers: readonly string[],
): string | undefined {
  if (!urls) {
    return undefined;
  }

  const matchedUrl = urls.find((candidateUrl) => {
    if (typeof candidateUrl.url !== "string") {
      return false;
    }

    if (typeof candidateUrl.label !== "string") {
      return false;
    }

    const normalizedLabel = candidateUrl.label.toLowerCase();
    return labelMatchers.some((matcher) => normalizedLabel.includes(matcher));
  });

  return matchedUrl?.url;
}

function findIcalUrlFromExtraFields(
  extraFields: DayliteExtraFields,
  labelMatchers: readonly string[],
): string | undefined {
  const matches = Object.entries(extraFields).find(([fieldKey]) => {
    const normalizedKey = fieldKey.toLowerCase();
    if (!normalizedKey.includes("ical")) {
      return false;
    }

    return labelMatchers.some((matcher) => normalizedKey.includes(matcher));
  });

  if (!matches) {
    return undefined;
  }

  const [, fieldValue] = matches;
  return fieldValue.value;
}

function extractDayliteObjectId(reference: string): string {
  const matchedIdentifier = reference.match(/\/(\d+)$/);
  return matchedIdentifier ? matchedIdentifier[1] : reference;
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
