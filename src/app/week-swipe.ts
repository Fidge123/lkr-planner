export type SwipeDirection = -1 | 1;

export type SwipeState =
  | { phase: "idle" }
  | {
      phase: "dragging";
      direction: SwipeDirection;
      /** How far the incoming week has been pulled in, in pixels, never negative. */
      offset: number;
      width: number;
    }
  | { phase: "settling"; direction: SwipeDirection; isCommitted: boolean }
  // The wheel keeps firing after the fingers leave the trackpad, so the tail of a
  // finished gesture must not start the next one.
  | { phase: "cooldown" };

export const swipeCommitRatio = 0.2;
export const swipeSettleMs = 250;
/** How long the wheel must stay quiet for the gesture to count as finished. */
export const swipeQuietMs = 120;

const activationDelta = 2;
const scrollEdgeTolerance = 1;

export interface WheelSample {
  deltaX: number;
  deltaY: number;
  scrollLeft: number;
  scrollWidth: number;
  clientWidth: number;
}

export interface SwipePreview {
  direction: SwipeDirection;
  /** Position of the incoming week relative to its own width: 100 is fully off screen, 0 fully over the current week. */
  translatePercent: number;
  isAnimating: boolean;
}

export function advanceSwipe(
  state: SwipeState,
  sample: WheelSample,
): SwipeState {
  if (state.phase === "settling" || state.phase === "cooldown") {
    return state;
  }
  const { deltaX, deltaY } = sample;
  if (
    Math.abs(deltaX) < activationDelta ||
    Math.abs(deltaX) <= Math.abs(deltaY)
  ) {
    return state;
  }

  if (state.phase === "dragging") {
    return {
      ...state,
      offset: clamp(state.offset + deltaX * state.direction, 0, state.width),
    };
  }

  const direction: SwipeDirection = deltaX > 0 ? 1 : -1;
  if (!isAtEdge(sample, direction)) {
    return state;
  }
  return {
    phase: "dragging",
    direction,
    offset: Math.min(Math.abs(deltaX), sample.clientWidth),
    width: sample.clientWidth,
  };
}

export function endSwipe(state: SwipeState): SwipeState {
  if (state.phase === "dragging") {
    return {
      phase: "settling",
      direction: state.direction,
      isCommitted: progressOf(state) >= swipeCommitRatio,
    };
  }
  if (state.phase === "cooldown") {
    return { phase: "idle" };
  }
  return state;
}

export function settleSwipe(state: SwipeState): SwipeState {
  return state.phase === "settling" ? { phase: "cooldown" } : state;
}

export function swipePreview(state: SwipeState): SwipePreview | null {
  if (state.phase === "dragging") {
    return {
      direction: state.direction,
      translatePercent: state.direction * 100 * (1 - progressOf(state)),
      isAnimating: false,
    };
  }
  if (state.phase === "settling") {
    return {
      direction: state.direction,
      translatePercent: state.isCommitted ? 0 : state.direction * 100,
      isAnimating: true,
    };
  }
  return null;
}

function progressOf(state: { offset: number; width: number }): number {
  return state.width > 0 ? clamp(state.offset / state.width, 0, 1) : 0;
}

function isAtEdge(sample: WheelSample, direction: SwipeDirection): boolean {
  return direction === 1
    ? sample.scrollLeft >=
        sample.scrollWidth - sample.clientWidth - scrollEdgeTolerance
    : sample.scrollLeft <= scrollEdgeTolerance;
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}
