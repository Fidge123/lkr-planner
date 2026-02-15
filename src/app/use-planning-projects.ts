import { useCallback, useEffect, useState } from "react";
import type { DayliteProjectRecord } from "../domain/planning";
import { loadDayliteProjects } from "../services/daylite-projects";
import type { PlanningGridProjectsState } from "./page";

export function usePlanningProjects(): PlanningGridProjectsState {
  const [projects, setProjects] = useState<DayliteProjectRecord[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const loadProjects = useCallback(async (forceRefresh: boolean) => {
    setIsLoading(true);

    try {
      const result = await loadDayliteProjects({ forceRefresh });
      setProjects(result.projects);
      setErrorMessage(result.errorMessage);
    } catch (error) {
      setErrorMessage(
        error instanceof Error
          ? error.message
          : "Die Daten konnten nicht von Daylite geladen werden.",
      );
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadProjects(false);
  }, [loadProjects]);

  const reloadProjects = useCallback(() => {
    void loadProjects(true);
  }, [loadProjects]);

  return { projects, isLoading, errorMessage, reloadProjects };
}
