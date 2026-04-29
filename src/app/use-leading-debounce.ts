import { useEffect, useRef, useState } from "react";

/**
 * Returns a debounced value that fires immediately on the leading edge
 * (first change after a quiet period) and again on the trailing edge
 * if additional changes arrived during the delay window.
 */
export function useLeadingDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState(value);
  const isDebouncing = useRef(false);

  useEffect(() => {
    if (!isDebouncing.current) {
      setDebouncedValue(value);
    }
    isDebouncing.current = true;

    const timer = setTimeout(() => {
      isDebouncing.current = false;
      setDebouncedValue(value);
    }, delay);

    return () => clearTimeout(timer);
  }, [value, delay]);

  return debouncedValue;
}
