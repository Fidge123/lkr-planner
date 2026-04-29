import { useCallback, useEffect, useRef, useState } from "react";
import type { Holiday } from "../generated/tauri";
import { commands } from "../generated/tauri";
import { useLeadingDebounce } from "./use-leading-debounce";

export interface HolidaysState {
  holidays: Holiday[];
  isLoading: boolean;
  errorMessage: string | null;
  reloadHolidays: () => void;
}

export function useHolidays(weekStart: string): HolidaysState {
  const debouncedWeekStart = useLeadingDebounce(weekStart, 200);
  const [holidays, setHolidays] = useState<Holiday[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const requestIdRef = useRef(0);

  const reloadHolidays = useCallback(() => {
    const id = ++requestIdRef.current;
    setIsLoading(true);
    void commands.getHolidaysForWeek(debouncedWeekStart).then((result) => {
      if (id !== requestIdRef.current) return;
      setIsLoading(false);
      if (result.status === "error") {
        setErrorMessage(result.error);
        setHolidays([]);
        return;
      }
      setHolidays(result.data);
      setErrorMessage(null);
    });
  }, [debouncedWeekStart]);

  useEffect(() => {
    reloadHolidays();
  }, [reloadHolidays]);

  return { holidays, isLoading, errorMessage, reloadHolidays };
}
