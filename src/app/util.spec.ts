import { beforeAll, describe, expect, it, setSystemTime } from "bun:test";
import { getWeekDays, isToday, toLocalISODate } from "./util";

describe("util", () => {
  beforeAll(() => {
    setSystemTime(new Date(2026, 0, 1, 12, 34, 56));
  });

  describe("getWeekDays", () => {
    it("should return the correct week days", () => {
      const weekDays = getWeekDays(0);
      expect(weekDays).toHaveLength(5);
      expect(weekDays[0].toISOString()).toBe("2025-12-29T00:00:00.000Z");
      expect(weekDays[4].toISOString()).toBe("2026-01-02T00:00:00.000Z");
    });

    it("should return the correct week days for a week offset", () => {
      const weekDays = getWeekDays(1);
      expect(weekDays[0].toISOString()).toBe("2026-01-05T00:00:00.000Z");
      expect(weekDays[4].toISOString()).toBe("2026-01-09T00:00:00.000Z");
    });

    it("should return the correct week days for a negative week offset", () => {
      const weekDays = getWeekDays(-4);
      expect(weekDays[0].toISOString()).toBe("2025-12-01T00:00:00.000Z");
      expect(weekDays[4].toISOString()).toBe("2025-12-05T00:00:00.000Z");
    });

    it("should produce dates at local midnight (hours/minutes/seconds are zero)", () => {
      for (const day of getWeekDays(0)) {
        expect(day.getHours()).toBe(0);
        expect(day.getMinutes()).toBe(0);
        expect(day.getSeconds()).toBe(0);
      }
    });
  });

  describe("toLocalISODate", () => {
    it("formats a date as yyyy-MM-dd using local time", () => {
      // new Date(year, month, date) constructs local midnight
      expect(toLocalISODate(new Date(2026, 0, 1))).toBe("2026-01-01");
      expect(toLocalISODate(new Date(2026, 11, 31))).toBe("2026-12-31");
      expect(toLocalISODate(new Date(2026, 3, 7))).toBe("2026-04-07");
    });
  });

  describe("isToday", () => {
    it("should return true for today", () => {
      expect(isToday(new Date())).toBe(true);
      expect(isToday(new Date(2026, 0, 1, 23, 59, 59))).toBe(true);
      expect(isToday(new Date(2026, 0, 1, 0, 0, 0))).toBe(true);
    });

    it("should return false for a different day", () => {
      expect(isToday(new Date(2025, 0, 1, 12, 34, 56))).toBe(false);
      expect(isToday(new Date(2026, 1, 1, 12, 34, 56))).toBe(false);
      expect(isToday(new Date(2026, 0, 2, 12, 34, 56))).toBe(false);
      expect(isToday(new Date(1970, 0, 1))).toBe(false);
    });
  });
});
