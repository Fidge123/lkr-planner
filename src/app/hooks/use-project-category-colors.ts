import { useEffect, useState } from "react";
import {
  loadProjectCategoryColors,
  type ProjectCategoryColors,
} from "../../services/daylite-categories";

export function useProjectCategoryColors(): ProjectCategoryColors {
  const [colors, setColors] = useState<ProjectCategoryColors>({});

  useEffect(() => {
    let cancelled = false;
    loadProjectCategoryColors().then((next) => {
      if (!cancelled) setColors(next);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  return colors;
}
