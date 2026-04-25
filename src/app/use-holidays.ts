import { useCallback, useEffect, useState } from "react";
import type { Holiday } from "../generated/tauri";
import { commands } from "../generated/tauri";

export interface HolidaysState {
  holidays: Holiday[];
  errorMessage: string | null;
}

export function useHolidays(weekStart: string): HolidaysState {
  const [holidays, setHolidays] = useState<Holiday[]>([]);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const loadHolidays = useCallback(async (ws: string) => {
    const result = await commands.getHolidaysForWeek(ws);
    if (result.status === "error") {
      setErrorMessage(result.error);
      setHolidays([]);
      return;
    }
    setHolidays(result.data);
    setErrorMessage(null);
  }, []);

  useEffect(() => {
    void loadHolidays(weekStart);
  }, [weekStart, loadHolidays]);

  return { holidays, errorMessage };
}
