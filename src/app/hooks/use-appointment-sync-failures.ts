import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { commands } from "../../generated/tauri";

export const appointmentSyncFailuresEvent = "daylite-appointment-sync-failures";

export interface AppointmentSyncFailuresState {
  /** ISO dates whose Daylite appointments could not be synchronized. */
  failedDates: string[];
  isRetrying: boolean;
  retry: () => Promise<void>;
}

export type AppointmentSyncFailuresListen = (
  event: string,
  handler: (event: { payload: string[] }) => void,
) => Promise<() => void>;

/**
 * The subscription only exists once `listen` resolves, so an unmount that races the
 * setup has to unlisten through the promise instead of a captured value.
 */
export function subscribeToAppointmentSyncFailures(
  listenFn: AppointmentSyncFailuresListen,
  onFailedDates: (dates: string[]) => void,
): () => void {
  const subscription = listenFn(appointmentSyncFailuresEvent, (event) =>
    onFailedDates(event.payload),
  );
  return () => {
    void subscription.then((unlisten) => unlisten()).catch(() => {});
  };
}

export function createRetryRunner(
  retryCommand: () => Promise<unknown>,
  setIsRetrying: (isRetrying: boolean) => void,
): () => Promise<void> {
  let isRunning = false;
  return async () => {
    if (isRunning) return;
    isRunning = true;
    setIsRetrying(true);
    try {
      await retryCommand();
    } finally {
      isRunning = false;
      setIsRetrying(false);
    }
  };
}

export function useAppointmentSyncFailures(): AppointmentSyncFailuresState {
  const [failedDates, setFailedDates] = useState<string[]>([]);
  const [isRetrying, setIsRetrying] = useState(false);
  const [retry] = useState(() =>
    createRetryRunner(commands.retryDayliteAppointmentSync, setIsRetrying),
  );

  useEffect(
    () => subscribeToAppointmentSyncFailures(listen, setFailedDates),
    [],
  );

  return { failedDates, isRetrying, retry };
}
