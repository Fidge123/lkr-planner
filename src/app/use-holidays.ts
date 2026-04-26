import { useCallback, useEffect, useState } from "react";
import type { Holiday } from "../generated/tauri";
import { commands } from "../generated/tauri";

export interface HolidaysState {
  holidays: Holiday[];
  errorMessage: string | null;
  reloadHolidays: () => void;
}

export function useHolidays(weekStart: string): HolidaysState {
  const [holidays, setHolidays] = useState<Holiday[]>([]);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const reloadHolidays = useCallback(() => {
    void commands.getHolidaysForWeek(weekStart).then((result) => {
      if (result.status === "error") {
        setErrorMessage(result.error);
        setHolidays([]);
        return;
      }
      setHolidays(result.data);
      setErrorMessage(null);
    });
  }, [weekStart]);

  useEffect(() => {
    reloadHolidays();
  }, [reloadHolidays]);

  return { holidays, errorMessage, reloadHolidays };
}
