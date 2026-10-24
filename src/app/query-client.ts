import { onlineManager, QueryClient } from "@tanstack/react-query";
import { returnSettleDelayMs } from "./hooks/use-calendar-auto-refresh";

/** Data stays fresh until a write or "Daten neu laden" invalidates it, and a failure surfaces at once with its reload action. */
export function createQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: {
        staleTime: Number.POSITIVE_INFINITY,
        retry: false,
        refetchOnWindowFocus: false,
        refetchOnReconnect: false,
        // A first attempt always runs; only a retry waits for the network.
        networkMode: "offlineFirst",
      },
    },
  });
}

export const queryClient = createQueryClient();

/** Wi-Fi and VPN report back before they carry traffic after wake, so a paused retry resumes only after the settle delay. */
export function installSettledOnlineManager(window: Window): void {
  onlineManager.setEventListener((setOnline) => {
    let settle: ReturnType<typeof setTimeout> | undefined;
    const online = () => {
      clearTimeout(settle);
      settle = setTimeout(() => setOnline(true), returnSettleDelayMs);
    };
    const offline = () => {
      clearTimeout(settle);
      setOnline(false);
    };
    setOnline(window.navigator.onLine);
    window.addEventListener("online", online);
    window.addEventListener("offline", offline);
    return () => {
      clearTimeout(settle);
      window.removeEventListener("online", online);
      window.removeEventListener("offline", offline);
    };
  });
}
