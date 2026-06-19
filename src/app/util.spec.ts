import {
  afterAll,
  beforeAll,
  describe,
  expect,
  it,
  setSystemTime,
} from "bun:test";
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

    it("returns 5 days from Monday to Friday when showWeekend is false", () => {
      const weekDays = getWeekDays(0, false);
      expect(weekDays).toHaveLength(5);
      expect(weekDays[0].getDay()).toBe(1); // Monday
      expect(weekDays[4].getDay()).toBe(5); // Friday
    });

    it("returns 7 days from Monday to Sunday when showWeekend is true", () => {
      const weekDays = getWeekDays(0, true);
      expect(weekDays).toHaveLength(7);
      expect(weekDays[0].getDay()).toBe(1); // Monday
      expect(weekDays[5].getDay()).toBe(6); // Saturday
      expect(weekDays[6].getDay()).toBe(0); // Sunday
    });
  });

  // Anchoring only differs on weekends, so these blocks pin the system clock to
  // a Saturday and a Sunday respectively.
  describe("getWeekDays weekend-aware anchoring", () => {
    describe("when today is a Saturday", () => {
      beforeAll(() => {
        // 2026-01-03 is a Saturday.
        setSystemTime(new Date(2026, 0, 3, 12, 0, 0));
      });
      afterAll(() => {
        setSystemTime(new Date(2026, 0, 1, 12, 34, 56));
      });

      it("anchors to the current week so today stays visible when showWeekend is on", () => {
        const weekDays = getWeekDays(0, true);
        expect(weekDays).toHaveLength(7);
        expect(weekDays[0].getDay()).toBe(1); // Monday
        expect(weekDays.map(toLocalISODate)).toContain("2026-01-03");
        expect(toLocalISODate(weekDays[5])).toBe("2026-01-03"); // Saturday is today
      });

      it("anchors to the upcoming Monday when showWeekend is off", () => {
        const weekDays = getWeekDays(0, false);
        expect(weekDays).toHaveLength(5);
        expect(toLocalISODate(weekDays[0])).toBe("2026-01-05");
        expect(weekDays.map(toLocalISODate)).not.toContain("2026-01-03");
      });
    });

    describe("when today is a Sunday", () => {
      beforeAll(() => {
        // 2026-01-04 is a Sunday.
        setSystemTime(new Date(2026, 0, 4, 12, 0, 0));
      });
      afterAll(() => {
        setSystemTime(new Date(2026, 0, 1, 12, 34, 56));
      });

      it("anchors to the current week so today stays visible when showWeekend is on", () => {
        const weekDays = getWeekDays(0, true);
        expect(weekDays).toHaveLength(7);
        expect(weekDays[0].getDay()).toBe(1); // Monday
        expect(weekDays.map(toLocalISODate)).toContain("2026-01-04");
        expect(toLocalISODate(weekDays[6])).toBe("2026-01-04"); // Sunday is today
      });

      it("anchors to the upcoming Monday when showWeekend is off", () => {
        const weekDays = getWeekDays(0, false);
        expect(weekDays).toHaveLength(5);
        expect(toLocalISODate(weekDays[0])).toBe("2026-01-05");
        expect(weekDays.map(toLocalISODate)).not.toContain("2026-01-04");
      });
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
