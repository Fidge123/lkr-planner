import { listen } from "@tauri-apps/api/event";
import { useEffect, useEffectEvent } from "react";
import { subscribeToTauriEvent } from "../tauri-event";

const reloadDataEvent = "reload-data";

export function useReloadDataMenu(onReload: () => void): void {
  const reload = useEffectEvent(onReload);

  useEffect(
    () => subscribeToTauriEvent(listen, reloadDataEvent, () => reload()),
    [],
  );
}
