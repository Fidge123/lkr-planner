import { describe, expect, it } from "bun:test";
import { normalizeOptionalString } from "./daylite-service-helpers";

describe("normalizeOptionalString", () => {
  it("trims non-empty string values", () => {
    expect(normalizeOptionalString("  wert  ")).toBe("wert");
  });

  it("returns undefined for blank and non-string values", () => {
    expect(normalizeOptionalString("   ")).toBeUndefined();
    expect(normalizeOptionalString(null)).toBeUndefined();
    expect(normalizeOptionalString(undefined)).toBeUndefined();
  });
});
