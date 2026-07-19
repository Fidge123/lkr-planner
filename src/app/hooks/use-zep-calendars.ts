import { useCallback, useRef, useState } from "react";
import type { ZepCalendar } from "../../generated/tauri";
import { discoverZepCalendars } from "../../services/zep";

export interface ZepCalendarsState {
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
  // Guards ensureLoaded against double-fetching: the isLoading state only
  // updates on the next render, the ref flips immediately.
  const isFetching = useRef(false);

  const reload = useCallback(() => {
    void (async () => {
      isFetching.current = true;
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
        isFetching.current = false;
        setIsLoading(false);
      }
    })();
  }, []);

  const ensureLoaded = useCallback(() => {
    if (calendars === null && !isFetching.current) {
      reload();
    }
  }, [calendars, reload]);

  return { calendars, isLoading, errorMessage, reload, ensureLoaded };
}
