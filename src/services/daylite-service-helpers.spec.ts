import { describe, expect, it } from "bun:test";
import {
  normalizeOptionalString,
  readDayliteApiErrorMessage,
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

  describe("readDayliteApiErrorMessage", () => {
    it("returns plain string errors unchanged", () => {
      const message = readDayliteApiErrorMessage("actual error", "fallback");

      expect(message).toBe("actual error");
    });

    it("returns userMessage from api errors when available", () => {
      const message = readDayliteApiErrorMessage(
        {
          userMessage: "user error",
          code: "UNAUTHORIZED",
          httpStatus: 401,
          technicalMessage: "tech msg",
        },
        "fallback",
      );

      expect(message).toBe("user error");
    });
  });
});
