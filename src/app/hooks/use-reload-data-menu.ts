import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";
import { subscribeToTauriEvent } from "../tauri-event";

const reloadDataEvent = "reload-data";

export function useReloadDataMenu(onReload: () => void): void {
  const onReloadRef = useRef(onReload);
  onReloadRef.current = onReload;

  useEffect(
    () =>
      subscribeToTauriEvent(listen, reloadDataEvent, () =>
        onReloadRef.current(),
      ),
    [],
  );
}
