import { useEffect, useEffectEvent } from "react";

/**
 * A dialog opened without showModal receives no cancel event, so Escape is caught on the document.
 * An input that handles Escape itself, such as by clearing its text, prevents the default to keep the dialog open.
 */
export function useEscapeKey(onEscape: () => void, isActive = true): void {
  const handleKeyDown = useEffectEvent((event: KeyboardEvent) => {
    if (event.key === "Escape" && !event.defaultPrevented) onEscape();
  });

  useEffect(() => {
    if (!isActive) return;
    const onKeyDown = (event: KeyboardEvent) => handleKeyDown(event);
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [isActive]);
}
