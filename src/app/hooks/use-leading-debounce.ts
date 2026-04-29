import { useEffect, useRef, useState } from "react";

export function useLeadingDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState(value);
  const isDebouncing = useRef(false);
  const hasPendingChange = useRef(false);
  const latestValue = useRef(value);

  useEffect(() => {
    latestValue.current = value;

    if (!isDebouncing.current) {
      setDebouncedValue(value);
      hasPendingChange.current = false;
    } else {
      hasPendingChange.current = true;
    }
    isDebouncing.current = true;

    const timer = setTimeout(() => {
      isDebouncing.current = false;
      if (hasPendingChange.current) {
        hasPendingChange.current = false;
        setDebouncedValue(latestValue.current);
      }
    }, delay);

    return () => clearTimeout(timer);
  }, [value, delay]);

  return debouncedValue;
}
