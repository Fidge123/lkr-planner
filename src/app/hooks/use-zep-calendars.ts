import { useQuery } from "@tanstack/react-query";
import type { ZepCalendar } from "../../generated/tauri";
import { discoverZepCalendars } from "../../services/zep";

export interface ZepCalendarsState {
  calendars: ZepCalendar[] | null;
  isLoading: boolean;
  errorMessage: string | null;
  reload: () => void;
}

/** Discovery starts the first time the dialog that needs it opens. */
export function useZepCalendars(isNeeded: boolean): ZepCalendarsState {
  const { data, isFetching, error, refetch } = useQuery({
    queryKey: ["zep-calendars"],
    queryFn: discoverZepCalendars,
    enabled: isNeeded,
  });
  return {
    calendars: data ?? null,
    isLoading: isFetching,
    errorMessage: error?.message ?? null,
    reload: () => void refetch(),
  };
}
