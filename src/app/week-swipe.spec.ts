import { describe, expect, it } from "bun:test";
import type { SwipeState, WheelSample } from "./week-swipe";
import {
  advanceSwipe,
  endSwipe,
  settleSwipe,
  swipeCommitRatio,
  swipePreview,
} from "./week-swipe";

const width = 1000;

/** A grid whose table fits, so both scroll edges are reached at once. */
function wheel(deltaX: number, overrides: Partial<WheelSample> = {}) {
  return {
    deltaX,
    deltaY: 0,
    scrollLeft: 0,
    scrollWidth: width,
    clientWidth: width,
    ...overrides,
  };
}

const idle: SwipeState = { phase: "idle" };

function dragging(direction: -1 | 1, offset: number): SwipeState {
  return { phase: "dragging", direction, offset, width };
}

describe("advanceSwipe", () => {
  it("should ignore a vertically dominant wheel event", () => {
    expect(advanceSwipe(idle, wheel(10, { deltaY: 40 }))).toBe(idle);
  });

  it("should ignore wheel noise below the activation delta", () => {
    expect(advanceSwipe(idle, wheel(1))).toBe(idle);
  });

  it("should start the next-week gesture at the right scroll edge", () => {
    expect(advanceSwipe(idle, wheel(30))).toEqual(dragging(1, 30));
  });

  it("should start the previous-week gesture at the left scroll edge", () => {
    expect(advanceSwipe(idle, wheel(-30))).toEqual(dragging(-1, 30));
  });

  it("should let the grid scroll while it is not yet at the right edge", () => {
    const sample = wheel(30, { scrollWidth: 1600, scrollLeft: 200 });
    expect(advanceSwipe(idle, sample)).toBe(idle);
  });

  it("should let the grid scroll while it is not yet at the left edge", () => {
    const sample = wheel(-30, { scrollWidth: 1600, scrollLeft: 200 });
    expect(advanceSwipe(idle, sample)).toBe(idle);
  });

  it("should start once the grid has reached the right edge", () => {
    const sample = wheel(30, { scrollWidth: 1600, scrollLeft: 600 });
    expect(advanceSwipe(idle, sample)).toEqual(dragging(1, 30));
  });

  it("should accumulate further pull in the same direction", () => {
    expect(advanceSwipe(dragging(1, 30), wheel(20))).toEqual(dragging(1, 50));
    expect(advanceSwipe(dragging(-1, 30), wheel(-20))).toEqual(
      dragging(-1, 50),
    );
  });

  it("should push the week back out when the swipe reverses", () => {
    expect(advanceSwipe(dragging(1, 50), wheel(-20))).toEqual(dragging(1, 30));
  });

  it("should keep the pulled distance within the grid width", () => {
    expect(advanceSwipe(dragging(1, 900), wheel(400))).toEqual(
      dragging(1, width),
    );
    expect(advanceSwipe(dragging(1, 30), wheel(-400))).toEqual(dragging(1, 0));
  });

  it("should keep a started gesture on its own axis", () => {
    const sample = wheel(20, { deltaY: 60 });
    expect(advanceSwipe(dragging(1, 30), sample)).toEqual(dragging(1, 30));
  });

  it("should swallow the momentum tail while the week settles", () => {
    const settling: SwipeState = {
      phase: "settling",
      direction: 1,
      isCommitted: true,
    };
    expect(advanceSwipe(settling, wheel(30))).toBe(settling);
  });

  it("should not start a second gesture during the cooldown", () => {
    const cooldown: SwipeState = { phase: "cooldown" };
    expect(advanceSwipe(cooldown, wheel(30))).toBe(cooldown);
  });
});

describe("endSwipe", () => {
  it("should snap back when the week was not pulled in far enough", () => {
    const state = dragging(1, width * swipeCommitRatio - 1);
    expect(endSwipe(state)).toEqual({
      phase: "settling",
      direction: 1,
      isCommitted: false,
    });
  });

  it("should slide the week over once the commit ratio is reached", () => {
    const state = dragging(-1, width * swipeCommitRatio);
    expect(endSwipe(state)).toEqual({
      phase: "settling",
      direction: -1,
      isCommitted: true,
    });
  });

  it("should release the cooldown once the wheel goes quiet", () => {
    expect(endSwipe({ phase: "cooldown" })).toEqual(idle);
  });

  it("should leave a settling week alone", () => {
    const settling: SwipeState = {
      phase: "settling",
      direction: 1,
      isCommitted: true,
    };
    expect(endSwipe(settling)).toBe(settling);
  });
});

describe("settleSwipe", () => {
  it("should enter the cooldown when the animation is done", () => {
    const settling: SwipeState = {
      phase: "settling",
      direction: 1,
      isCommitted: false,
    };
    expect(settleSwipe(settling)).toEqual({ phase: "cooldown" });
  });

  it("should leave every other phase alone", () => {
    expect(settleSwipe(idle)).toBe(idle);
  });
});

describe("swipePreview", () => {
  it("should render nothing without a gesture", () => {
    expect(swipePreview(idle)).toBeNull();
    expect(swipePreview({ phase: "cooldown" })).toBeNull();
  });

  it("should hold the next week off the right edge and pull it in", () => {
    expect(swipePreview(dragging(1, 0))).toEqual({
      direction: 1,
      translatePercent: 100,
      isAnimating: false,
    });
    expect(swipePreview(dragging(1, 250))).toEqual({
      direction: 1,
      translatePercent: 75,
      isAnimating: false,
    });
  });

  it("should hold the previous week off the left edge", () => {
    expect(swipePreview(dragging(-1, 250))).toEqual({
      direction: -1,
      translatePercent: -75,
      isAnimating: false,
    });
  });

  it("should animate a committed week all the way over the current one", () => {
    const settling: SwipeState = {
      phase: "settling",
      direction: 1,
      isCommitted: true,
    };
    expect(swipePreview(settling)).toEqual({
      direction: 1,
      translatePercent: 0,
      isAnimating: true,
    });
  });

  it("should animate a rejected week back off the edge it came from", () => {
    const settling: SwipeState = {
      phase: "settling",
      direction: -1,
      isCommitted: false,
    };
    expect(swipePreview(settling)).toEqual({
      direction: -1,
      translatePercent: -100,
      isAnimating: true,
    });
  });

  it("should not divide by a zero width", () => {
    const state: SwipeState = {
      phase: "dragging",
      direction: 1,
      offset: 0,
      width: 0,
    };
    expect(swipePreview(state)).toEqual({
      direction: 1,
      translatePercent: 100,
      isAnimating: false,
    });
  });
});
