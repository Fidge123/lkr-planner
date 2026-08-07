import { useEffect, useState } from "react";
import {
  loadProjectCategoryColors,
  type ProjectCategoryColors,
} from "../../services/daylite-categories";

export function useProjectCategoryColors(
  isActive: boolean,
): ProjectCategoryColors {
  const [colors, setColors] = useState<ProjectCategoryColors>({});

  useEffect(() => {
    if (!isActive) return;
    let cancelled = false;
    loadProjectCategoryColors().then((next) => {
      if (!cancelled) setColors(next);
    });
    return () => {
      cancelled = true;
    };
  }, [isActive]);

  return colors;
}
