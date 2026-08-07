import type { RefObject } from "react";
import { useCallback, useEffect, useRef, useState } from "react";
import type { SwipeDirection, SwipePreview, SwipeState } from "../week-swipe";
import {
  advanceSwipe,
  endSwipe,
  settleSwipe,
  swipePreview,
  swipeQuietMs,
  swipeSettleMs,
} from "../week-swipe";

interface UseWeekSwipeArgs {
  containerRef: RefObject<HTMLElement | null>;
  onNavigate: (direction: SwipeDirection) => void;
  isDisabled: boolean;
}

export function useWeekSwipe({
  containerRef,
  onNavigate,
  isDisabled,
}: UseWeekSwipeArgs): SwipePreview | null {
  const [state, setState] = useState<SwipeState>({ phase: "idle" });
  const stateRef = useRef(state);
  stateRef.current = state;
  const onNavigateRef = useRef(onNavigate);
  onNavigateRef.current = onNavigate;

  const quietTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const scheduleQuiet = useCallback(() => {
    if (quietTimer.current !== null) clearTimeout(quietTimer.current);
    quietTimer.current = setTimeout(() => {
      quietTimer.current = null;
      setState((current) => endSwipe(current));
    }, swipeQuietMs);
  }, []);

  useEffect(() => {
    return () => {
      if (quietTimer.current !== null) clearTimeout(quietTimer.current);
    };
  }, []);

  useEffect(() => {
    const container = containerRef.current;
    if (!container || isDisabled) return;

    const onWheel = (event: WheelEvent) => {
      const next = advanceSwipe(stateRef.current, {
        deltaX: event.deltaX,
        deltaY: event.deltaY,
        scrollLeft: container.scrollLeft,
        scrollWidth: container.scrollWidth,
        clientWidth: container.clientWidth,
      });
      // Only a gesture that is running (or just ran) owns the wheel; anything else stays the grid's own scroll.
      if (next.phase === "idle") return;
      event.preventDefault();
      stateRef.current = next;
      setState(next);
      scheduleQuiet();
    };

    container.addEventListener("wheel", onWheel, { passive: false });
    return () => container.removeEventListener("wheel", onWheel);
  }, [containerRef, isDisabled, scheduleQuiet]);

  // An appointment drag takes over week navigation through its own edge hovering.
  useEffect(() => {
    if (isDisabled) setState({ phase: "idle" });
  }, [isDisabled]);

  useEffect(() => {
    if (state.phase !== "settling") return;
    const timer = setTimeout(() => {
      if (state.isCommitted) onNavigateRef.current(state.direction);
      setState(settleSwipe(state));
    }, swipeSettleMs);
    return () => clearTimeout(timer);
  }, [state]);

  useEffect(() => {
    if (state.phase === "cooldown") scheduleQuiet();
  }, [state, scheduleQuiet]);

  return swipePreview(state);
}
