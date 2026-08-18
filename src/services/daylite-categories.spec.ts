import { beforeEach, describe, expect, it, mock } from "bun:test";
import { mockCommands } from "../test/mock-commands";

type CategoryColorsResult =
  | { status: "ok"; data: Record<string, string> }
  | { status: "error"; error: string };

const backendColors = { Bau: "#8bc34a", Überfällig: "#ff5722" };

const mockCategoryColors = mock(
  (): Promise<CategoryColorsResult> =>
    Promise.resolve({ status: "ok", data: { ...backendColors } }),
);

const failsOnce = () =>
  mockCategoryColors.mockImplementationOnce(() =>
    Promise.resolve({ status: "error", error: "Daylite nicht erreichbar" }),
  );

mockCommands({ dayliteProjectCategoryColors: mockCategoryColors });

const {
  loadProjectCategoryColors,
  projectCategoryColor,
  resetProjectCategoryColors,
  subscribeProjectCategoryColors,
} = await import("./daylite-categories");

describe("loadProjectCategoryColors", () => {
  beforeEach(() => {
    resetProjectCategoryColors();
    mockCategoryColors.mockClear();
  });

  it("returns the category colors from the backend", async () => {
    expect(await loadProjectCategoryColors()).toEqual(backendColors);
  });

  it("keeps the colors for the session instead of asking again", async () => {
    await loadProjectCategoryColors();
    await loadProjectCategoryColors();

    expect(mockCategoryColors).toHaveBeenCalledTimes(1);
  });

  it("asks again after a reset", async () => {
    await loadProjectCategoryColors();
    resetProjectCategoryColors();
    await loadProjectCategoryColors();

    expect(mockCategoryColors).toHaveBeenCalledTimes(2);
  });

  it("falls back to no colors when the command fails", async () => {
    failsOnce();

    expect(await loadProjectCategoryColors()).toEqual({});
  });

  it("retries after a failure instead of caching the empty result", async () => {
    failsOnce();

    await loadProjectCategoryColors();

    expect(await loadProjectCategoryColors()).toEqual(backendColors);
  });
});

describe("subscribeProjectCategoryColors", () => {
  beforeEach(() => {
    resetProjectCategoryColors();
    mockCategoryColors.mockClear();
  });

  it("notifies subscribers when the colors are reset", () => {
    let notifications = 0;
    const unsubscribe = subscribeProjectCategoryColors(() => {
      notifications += 1;
    });

    resetProjectCategoryColors();
    resetProjectCategoryColors();
    unsubscribe();
    resetProjectCategoryColors();

    expect(notifications).toBe(2);
  });

  it("lets a notified subscriber load the refreshed colors", async () => {
    let reloads = 0;
    const unsubscribe = subscribeProjectCategoryColors(() => {
      void loadProjectCategoryColors();
      reloads += 1;
    });
    await loadProjectCategoryColors();

    resetProjectCategoryColors();
    unsubscribe();

    expect(reloads).toBe(1);
    expect(mockCategoryColors).toHaveBeenCalledTimes(2);
  });
});

describe("projectCategoryColor", () => {
  const colors = { Bau: "#8bc34a" };

  it("resolves the color of a known category", () => {
    expect(projectCategoryColor(colors, "Bau")).toBe("#8bc34a");
  });

  it("ignores surrounding whitespace", () => {
    expect(projectCategoryColor(colors, " Bau ")).toBe("#8bc34a");
  });

  it("returns null for a project without a category", () => {
    expect(projectCategoryColor(colors, null)).toBeNull();
    expect(projectCategoryColor(colors, "  ")).toBeNull();
  });

  it("returns null for a category that has no color", () => {
    expect(projectCategoryColor(colors, "Wartung")).toBeNull();
  });
});
