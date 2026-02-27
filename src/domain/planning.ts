import type {
  DayliteContactUrl,
  PlanningContactRecord,
  PlanningProjectRecord,
  PlanningProjectStatus,
} from "../generated/tauri";

interface AssignmentPeriod {
  startDate: string;
  endDate: string;
}

export interface Assignment {
  id: string;
  employeeId: string;
  projectId: string;
  period: AssignmentPeriod;
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

export type DayliteContactRecord = PlanningContactRecord & {
  keywords?: string[];
  addresses?: DayliteAddress[];
};

export function isDayliteProjectRecord(
  value: unknown,
): value is PlanningProjectRecord {
  if (!isObject(value)) {
    return false;
  }

  if (typeof value.self !== "string" || typeof value.name !== "string") {
    return false;
  }

  if (!isPlanningProjectStatus(value.status)) {
    return false;
  }

  if (!Array.isArray(value.keywords)) {
    return false;
  }

  if (!value.keywords.every((keyword) => typeof keyword === "string")) {
    return false;
  }

  return true;
}

export function isDayliteContactRecord(
  value: unknown,
): value is DayliteContactRecord {
  if (
    !isObject(value) ||
    typeof value.self !== "string" ||
    !Array.isArray(value.urls) ||
    !value.urls.every(isDayliteUrlRecord)
  ) {
    return false;
  }

  if ("full_name" in value && value.full_name !== undefined) {
    if (value.full_name !== null && typeof value.full_name !== "string") {
      return false;
    }
  }

  if ("nickname" in value && value.nickname !== undefined) {
    if (value.nickname !== null && typeof value.nickname !== "string") {
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

  return true;
}

export function getDayliteContactDisplayName(
  record: DayliteContactRecord,
): string {
  const nickname = record?.nickname?.trim();
  if (nickname) {
    return nickname;
  }

  const fullName = record?.full_name?.trim();
  if (fullName) {
    return fullName;
  }

  return "Unbenannter Kontakt";
}

export function getPrimaryAddressFromContact(
  record: DayliteContactRecord,
): DayliteAddress | null {
  return record.addresses?.[0] ?? null;
}

export function getPrimaryIcalUrlFromContact(
  record: DayliteContactRecord,
): string {
  return findIcalUrlFromUrls(record.urls, matchesPrimaryIcalLabel) ?? "";
}

export function getAbsenceIcalUrlFromContact(
  record: DayliteContactRecord,
): string {
  return findIcalUrlFromUrls(record.urls, matchesAbsenceIcalLabel) ?? "";
}

export function upsertDayliteContactIcalUrls(
  urls: DayliteContactUrl[] | undefined,
  primaryIcalUrl: string,
  absenceIcalUrl: string,
): DayliteContactUrl[] {
  const preservedUrls = (urls ?? []).filter((candidateUrl) => {
    const label = normalizeLabel(candidateUrl.label);
    if (!label) {
      return true;
    }

    return !matchesPrimaryIcalLabel(label) && !matchesAbsenceIcalLabel(label);
  });

  const nextUrls = [...preservedUrls];
  const normalizedPrimaryIcalUrl = normalizeUrl(primaryIcalUrl);
  const normalizedAbsenceIcalUrl = normalizeUrl(absenceIcalUrl);

  if (normalizedPrimaryIcalUrl) {
    nextUrls.push({
      label: "Einsatz iCal",
      url: normalizedPrimaryIcalUrl,
    });
  }

  if (normalizedAbsenceIcalUrl) {
    nextUrls.push({
      label: "Abwesenheit iCal",
      url: normalizedAbsenceIcalUrl,
    });
  }

  return nextUrls;
}

function findIcalUrlFromUrls(
  urls: DayliteContactUrl[] | undefined,
  matcher: (normalizedLabel: string) => boolean,
): string | undefined {
  if (!urls) {
    return undefined;
  }

  const matchedUrl = urls.find((candidateUrl) => {
    if (typeof candidateUrl.url !== "string") {
      return false;
    }

    const normalizedLabel = normalizeLabel(candidateUrl.label);
    if (!normalizedLabel) {
      return false;
    }

    return matcher(normalizedLabel);
  });

  return normalizeUrl(matchedUrl?.url);
}

function matchesPrimaryIcalLabel(normalizedLabel: string): boolean {
  return (
    normalizedLabel.includes("einsatz") || normalizedLabel.includes("termine")
  );
}

function matchesAbsenceIcalLabel(normalizedLabel: string): boolean {
  return (
    normalizedLabel.includes("abwesenheit") ||
    normalizedLabel.includes("fehlzeiten")
  );
}

function normalizeLabel(label: string | null | undefined): string | undefined {
  if (typeof label !== "string") {
    return undefined;
  }

  const normalized = label.trim().toLowerCase();
  return normalized.length > 0 ? normalized : undefined;
}

function normalizeUrl(url: string | null | undefined): string | undefined {
  if (typeof url !== "string") {
    return undefined;
  }

  const normalized = url.trim();
  return normalized.length > 0 ? normalized : undefined;
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

const planningProjectStatuses: Set<PlanningProjectStatus> = new Set([
  "new_status",
  "in_progress",
  "done",
  "abandoned",
  "cancelled",
  "deferred",
]);

function isPlanningProjectStatus(
  value: unknown,
): value is PlanningProjectStatus {
  return (
    typeof value === "string" &&
    planningProjectStatuses.has(value as PlanningProjectStatus)
  );
}

function isDayliteUrlRecord(value: unknown): value is DayliteContactUrl {
  if (!isObject(value)) {
    return false;
  }

  if (!isNullableString(value.label)) {
    return false;
  }

  if (!isNullableString(value.url)) {
    return false;
  }

  if (!isNullableString(value.note)) {
    return false;
  }

  return true;
}

function isNullableString(value: unknown): value is string | null | undefined {
  return value === undefined || value === null || typeof value === "string";
}
