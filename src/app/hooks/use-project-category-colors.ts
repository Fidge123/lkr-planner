import { useEffect, useState } from "react";
import {
  loadProjectCategoryColors,
  type ProjectCategoryColors,
  subscribeProjectCategoryColors,
} from "../../services/daylite-categories";

export function useProjectCategoryColors(): ProjectCategoryColors {
  const [colors, setColors] = useState<ProjectCategoryColors>({});

  useEffect(() => {
    let cancelled = false;
    const load = () => {
      loadProjectCategoryColors().then((next) => {
        if (!cancelled) setColors(next);
      });
    };

    load();
    const unsubscribe = subscribeProjectCategoryColors(load);
    return () => {
      cancelled = true;
      unsubscribe();
    };
  }, []);

  return colors;
}
