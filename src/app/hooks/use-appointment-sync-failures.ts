import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { commands } from "../../generated/tauri";
import { subscribeToTauriEvent } from "../tauri-event";

const appointmentSyncFailuresEvent = "daylite-appointment-sync-failures";

export function useAppointmentSyncFailures() {
  const [failedDates, setFailedDates] = useState<string[]>([]);
  const [isRetrying, setIsRetrying] = useState(false);

  useEffect(
    () =>
      subscribeToTauriEvent<string[]>(
        listen,
        appointmentSyncFailuresEvent,
        setFailedDates,
      ),
    [],
  );

  const retry = async () => {
    setIsRetrying(true);
    try {
      await commands.retryDayliteAppointmentSync();
    } finally {
      setIsRetrying(false);
    }
  };

  return { failedDates, isRetrying, retry };
}
