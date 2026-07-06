import { describe, expect, it } from "bun:test";
import {
  normalizeOptionalString,
  unwrapCommandResult,
} from "./daylite-service-helpers";

describe("daylite service helpers", () => {
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

  describe("unwrapCommandResult", () => {
    it("returns the data on ok results", () => {
      const value = unwrapCommandResult(
        { status: "ok", data: "wert" },
        "fallback",
      );

      expect(value).toBe("wert");
    });

    it("throws plain string errors unchanged", () => {
      expect(() =>
        unwrapCommandResult(
          { status: "error", error: "actual error" },
          "fallback",
        ),
      ).toThrow("actual error");
    });

    it("throws the userMessage from api error objects when available", () => {
      expect(() =>
        unwrapCommandResult(
          {
            status: "error",
            error: {
              userMessage: "user error",
              code: "UNAUTHORIZED",
              httpStatus: 401,
              technicalMessage: "tech msg",
            },
          },
          "fallback",
        ),
      ).toThrow("user error");
    });

    it("throws the fallback message when the error has no userMessage", () => {
      expect(() =>
        unwrapCommandResult(
          { status: "error", error: { code: "UNAUTHORIZED" } },
          "fallback",
        ),
      ).toThrow("fallback");
    });
  });
});
