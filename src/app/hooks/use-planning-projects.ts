import { useCallback, useEffect, useRef, useState } from "react";
import type { PlanningProjectRecord } from "../../generated/tauri";
import { loadDayliteProjects } from "../../services/daylite-projects";
import type { PlanningGridProjectsState } from "../page";

export function usePlanningProjects(): PlanningGridProjectsState {
  const requestIdRef = useRef(0);
  const [projects, setProjects] = useState<PlanningProjectRecord[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const loadProjects = useCallback(async (forceRefresh: boolean) => {
    const id = ++requestIdRef.current;
    setIsLoading(true);

    try {
      const result = await loadDayliteProjects({ forceRefresh });
      if (id !== requestIdRef.current) return;
      setProjects(result.projects);
      setErrorMessage(result.errorMessage ?? null);
    } catch (error) {
      if (id !== requestIdRef.current) return;
      setErrorMessage(
        error instanceof Error
          ? error.message
          : "Die Daten konnten nicht von Daylite geladen werden.",
      );
    } finally {
      if (id === requestIdRef.current) {
        setIsLoading(false);
      }
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
