import { useCallback, useEffect, useRef, useState } from "react";
import type { Holiday } from "../../generated/tauri";
import { loadHolidaysForWeek } from "../../services/holidays";

export interface HolidaysState {
  holidays: Holiday[];
  isLoading: boolean;
  errorMessage: string | null;
  reloadHolidays: () => void;
}

export function useHolidays(weekStart: string): HolidaysState {
  const [holidays, setHolidays] = useState<Holiday[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const requestIdRef = useRef(0);

  const reloadHolidays = useCallback(() => {
    const id = ++requestIdRef.current;
    setIsLoading(true);
    void loadHolidaysForWeek(weekStart).then(
      (loaded) => {
        if (id !== requestIdRef.current) return;
        setIsLoading(false);
        setHolidays(loaded);
        setErrorMessage(null);
      },
      (error: Error) => {
        if (id !== requestIdRef.current) return;
        setIsLoading(false);
        setErrorMessage(error.message);
        setHolidays([]);
      },
    );
  }, [weekStart]);

  useEffect(() => {
    reloadHolidays();
  }, [reloadHolidays]);

  return { holidays, isLoading, errorMessage, reloadHolidays };
}
