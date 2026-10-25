import { describe, expect, it } from "bun:test";
import { projectCategoryColor } from "./daylite-categories";

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
