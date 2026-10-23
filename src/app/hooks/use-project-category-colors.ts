import { queryOptions, useQuery } from "@tanstack/react-query";
import {
  loadProjectCategoryColors,
  type ProjectCategoryColors,
} from "../../services/daylite-categories";

export const categoryColorsQuery = queryOptions({
  queryKey: ["category-colors"],
  queryFn: loadProjectCategoryColors,
});

/** A missing color is cosmetic, so a failed load shows no colors and the next mount tries again. */
export function useProjectCategoryColors(): ProjectCategoryColors {
  return useQuery(categoryColorsQuery).data ?? {};
}
