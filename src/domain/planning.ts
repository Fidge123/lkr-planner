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

type DayliteExtraFieldValue = {
  value?: string;
};

type DayliteExtraFields = Record<string, DayliteExtraFieldValue>;

export interface DayliteContactRecord {
  self: string;
  full_name?: string;
  nickname?: string;
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
  return record?.nickname ?? record?.full_name ?? "";
}

export function getPrimaryAddressFromContact(
  record: DayliteContactRecord,
): DayliteAddress | null {
  return record.addresses?.[0] ?? null;
}

export function getPrimaryIcalUrlFromContact(
  record: DayliteContactRecord,
): string {
  const extraFields = parseExtraFields(record.extra_fields);
  return (
    findIcalUrlFromUrls(record.urls, "Termine") ??
    findIcalUrlFromExtraFields(extraFields, "Termine") ??
    ""
  );
}

export function getAbsenceIcalUrlFromContact(
  record: DayliteContactRecord,
): string {
  const extraFields = parseExtraFields(record.extra_fields);
  return (
    findIcalUrlFromUrls(record.urls, "Fehlzeiten") ??
    findIcalUrlFromExtraFields(extraFields, "Fehlzeiten") ??
    ""
  );
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
  matcher: string,
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
    return normalizedLabel.includes(matcher.toLowerCase());
  });

  return matchedUrl?.url;
}

function findIcalUrlFromExtraFields(
  extraFields: DayliteExtraFields,
  matcher: string,
): string | undefined {
  const matches = Object.entries(extraFields).find(([fieldKey]) => {
    const normalizedKey = fieldKey.toLowerCase();
    if (!normalizedKey.includes("ical")) {
      return false;
    }

    return normalizedKey.includes(matcher.toLowerCase());
  });

  if (!matches) {
    return undefined;
  }

  const [, fieldValue] = matches;
  return fieldValue.value;
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
