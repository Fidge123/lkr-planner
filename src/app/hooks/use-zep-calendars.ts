import { useCallback, useState } from "react";
import type { ZepCalendar } from "../../generated/tauri";
import { discoverZepCalendars } from "../../services/zep";

export interface ZepCalendarsState {
  // Session cache, not persisted across restarts: null means "not yet
  // fetched", [] means "fetched but empty".
  calendars: ZepCalendar[] | null;
  isLoading: boolean;
  errorMessage: string | null;
  reload: () => void;
  ensureLoaded: () => void;
}

export function useZepCalendars(): ZepCalendarsState {
  const [calendars, setCalendars] = useState<ZepCalendar[] | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const reload = useCallback(() => {
    void (async () => {
      setIsLoading(true);
      setErrorMessage(null);
      try {
        setCalendars(await discoverZepCalendars());
      } catch (error) {
        setErrorMessage(
          error instanceof Error
            ? error.message
            : "Die ZEP-Kalender konnten nicht geladen werden.",
        );
      } finally {
        setIsLoading(false);
      }
    })();
  }, []);

  const ensureLoaded = useCallback(() => {
    if (calendars === null && !isLoading) {
      reload();
    }
  }, [calendars, isLoading, reload]);

  return { calendars, isLoading, errorMessage, reload, ensureLoaded };
}
