import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

export const reloadDataEvent = "reload-data";

export type ReloadDataListen = (
  event: string,
  handler: () => void,
) => Promise<() => void>;

/**
 * The subscription only exists once `listen` resolves, so an unmount that races the
 * setup has to unlisten through the promise instead of a captured value.
 */
export function subscribeToReloadDataMenu(
  listenFn: ReloadDataListen,
  onReload: () => void,
): () => void {
  const subscription = listenFn(reloadDataEvent, onReload);
  return () => {
    void subscription.then((unlisten) => unlisten()).catch(() => {});
  };
}

/** Runs `onReload` when the "Daten neu laden" menu item is chosen. */
export function useReloadDataMenu(onReload: () => void): void {
  const onReloadRef = useRef(onReload);
  onReloadRef.current = onReload;

  useEffect(
    () => subscribeToReloadDataMenu(listen, () => onReloadRef.current()),
    [],
  );
}
