import type { DayliteProjectRecord } from "../domain/planning";

export interface StandardFilter {
  pipelines: string[];
  columns: string[];
  categories: string[];
  exclusionStatuses: string[];
}

export const defaultStandardFilter: StandardFilter = {
  pipelines: ["Aufträge"],
  columns: ["Vorbereitung", "Durchführung"],
  categories: ["Überfällig", "Liefertermin bekannt"],
  exclusionStatuses: ["Done"],
};

export function applyStandardFilter(
  projects: DayliteProjectRecord[],
  filter: StandardFilter = defaultStandardFilter,
): DayliteProjectRecord[] {
  return projects.filter((project) => matchesStandardFilter(project, filter));
}

export function matchesStandardFilter(
  project: DayliteProjectRecord,
  filter: StandardFilter = defaultStandardFilter,
): boolean {
  if (matchesExclusionStatus(project.status, filter.exclusionStatuses)) {
    return false;
  }

  return (
    matchesPipelineRule(project, filter.pipelines, filter.columns) ||
    matchesCategoryRule(project, filter.categories)
  );
}

function matchesPipelineRule(
  project: DayliteProjectRecord,
  pipelines: string[],
  columns: string[],
): boolean {
  const normalizedKeywords = normalizeStringArray(project.keywords);
  if (normalizedKeywords.length === 0) {
    return false;
  }

  const pipelineMatches = normalizeStringArray(pipelines).some((pipeline) =>
    normalizedKeywords.includes(pipeline),
  );
  const columnMatches = normalizeStringArray(columns).some((column) =>
    normalizedKeywords.includes(column),
  );

  return pipelineMatches && columnMatches;
}

function matchesCategoryRule(
  project: DayliteProjectRecord,
  categories: string[],
): boolean {
  const normalizedCategory = normalizeString(project.category);
  if (!normalizedCategory) {
    return false;
  }

  return normalizeStringArray(categories).includes(normalizedCategory);
}

function matchesExclusionStatus(
  status: DayliteProjectRecord["status"],
  exclusionStatuses: string[],
): boolean {
  const normalizedStatus = normalizeString(status);
  if (!normalizedStatus) {
    return false;
  }

  return normalizeStringArray(exclusionStatuses).includes(normalizedStatus);
}

function normalizeStringArray(values: string[] | undefined): string[] {
  if (!Array.isArray(values)) {
    return [];
  }

  return values
    .map(normalizeString)
    .filter((value): value is string => typeof value === "string");
}

function normalizeString(value: string | undefined): string | undefined {
  if (typeof value !== "string") {
    return undefined;
  }

  const normalized = value.trim().toLowerCase();
  return normalized.length > 0 ? normalized : undefined;
}
