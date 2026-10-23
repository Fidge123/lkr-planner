import { useQuery } from "@tanstack/react-query";
import type { Holiday } from "../../generated/tauri";
import { loadHolidaysForWeek } from "../../services/holidays";

export interface HolidaysState {
  holidays: Holiday[];
  errorMessage: string | null;
  reloadHolidays: () => void;
}

export function useHolidays(weekStart: string): HolidaysState {
  const { data, error, refetch } = useQuery({
    queryKey: ["holidays", weekStart],
    queryFn: () => loadHolidaysForWeek(weekStart),
  });
  return {
    holidays: data ?? [],
    errorMessage: error?.message ?? null,
    reloadHolidays: () => void refetch(),
  };
}
