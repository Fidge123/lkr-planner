import { queryOptions, useQuery } from "@tanstack/react-query";
import { loadTelemetrySettings } from "../../services/telemetry";

export const telemetrySettingsQuery = queryOptions({
  queryKey: ["telemetry-settings"],
  queryFn: loadTelemetrySettings,
});

export function useTelemetrySettings() {
  const { data, isPending } = useQuery(telemetrySettingsQuery);
  return { isLoaded: !isPending, isEnabled: data ?? false };
}
