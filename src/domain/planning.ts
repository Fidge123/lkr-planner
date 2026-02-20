export interface AssignmentPeriod {
  startDate: string;
  endDate: string;
}

export interface Assignment {
  id: string;
  employeeId: string;
  projectId: string;
  period: AssignmentPeriod;
}

export interface DayliteProjectRecord {
  self: string;
  name: string;
  status:
    | "new_status"
    | "in_progress"
    | "done"
    | "abandoned"
    | "cancelled"
    | "deferred";
  category?: string;
  keywords?: string[];
  due?: string;
  started?: string;
  completed?: string;
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

export interface DayliteContactRecord {
  self: string;
  full_name?: string;
  nickname?: string;
  category?: string;
  keywords?: string[];
  urls?: DayliteUrl[];
  addresses?: DayliteAddress[];
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

  if ("full_name" in value && value.full_name !== undefined) {
    if (typeof value.full_name !== "string") {
      return false;
    }
  }

  if ("nickname" in value && value.nickname !== undefined) {
    if (typeof value.nickname !== "string") {
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
  urls: DayliteUrl[] | undefined,
  primaryIcalUrl: string,
  absenceIcalUrl: string,
): DayliteUrl[] {
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
  urls: DayliteUrl[] | undefined,
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

function normalizeLabel(label: string | undefined): string | undefined {
  if (typeof label !== "string") {
    return undefined;
  }

  const normalized = label.trim().toLowerCase();
  return normalized.length > 0 ? normalized : undefined;
}

function normalizeUrl(url: string | undefined): string | undefined {
  if (typeof url !== "string") {
    return undefined;
  }

  const normalized = url.trim();
  return normalized.length > 0 ? normalized : undefined;
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
