import { loadDayliteProjects } from "../../services/daylite-projects";
import type { PlanningGridProjectsState } from "../page";
import { useCachedResource } from "./use-cached-resource";

export function usePlanningProjects(): PlanningGridProjectsState {
  const { data, isLoading, errorMessage, reload } = useCachedResource({
    load: loadDayliteProjects,
    errorMessage: "Die Daten konnten nicht von Daylite geladen werden.",
  });

  return {
    projects: data,
    isLoading,
    errorMessage,
    reloadProjects: reload,
  };
}
