import {
  afterAll,
  beforeAll,
  describe,
  expect,
  it,
  setSystemTime,
} from "bun:test";
import {
  getIsoWeek,
  getWeekDays,
  getWeekStart,
  isToday,
  shiftWeekDays,
  toLocalISODate,
} from "./util";

describe("util", () => {
  beforeAll(() => {
    setSystemTime(new Date(2026, 0, 1, 12, 34, 56));
  });

  describe("getWeekDays", () => {
    // Asserted as local calendar dates, not UTC instants: getWeekDays builds
    // local-midnight dates, so toISOString would shift them a day east of UTC
    // and pin these tests to a UTC-only test runner.
    it("should return the correct week days", () => {
      const weekDays = getWeekDays(0);
      expect(weekDays).toHaveLength(5);
      expect(toLocalISODate(weekDays[0])).toBe("2025-12-29");
      expect(toLocalISODate(weekDays[4])).toBe("2026-01-02");
    });

    it("should return the correct week days for a week offset", () => {
      const weekDays = getWeekDays(1);
      expect(toLocalISODate(weekDays[0])).toBe("2026-01-05");
      expect(toLocalISODate(weekDays[4])).toBe("2026-01-09");
    });

    it("should return the correct week days for a negative week offset", () => {
      const weekDays = getWeekDays(-4);
      expect(toLocalISODate(weekDays[0])).toBe("2025-12-01");
      expect(toLocalISODate(weekDays[4])).toBe("2025-12-05");
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

  describe("getWeekDays weekend-aware anchoring", () => {
    describe("when today is a Saturday", () => {
      beforeAll(() => {
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
      expect(toLocalISODate(new Date(2026, 0, 1))).toBe("2026-01-01");
      expect(toLocalISODate(new Date(2026, 11, 31))).toBe("2026-12-31");
      expect(toLocalISODate(new Date(2026, 3, 7))).toBe("2026-04-07");
    });
  });

  describe("getWeekStart", () => {
    it("covers every displayed day in the backend's seven-day window", () => {
      for (const showWeekend of [false, true]) {
        for (const offset of [-1, 0, 1]) {
          const [y, m, d] = getWeekStart(offset, showWeekend)
            .split("-")
            .map(Number);
          const start = new Date(y, m - 1, d);
          const end = new Date(y, m - 1, d + 7);
          for (const day of getWeekDays(offset, showWeekend)) {
            expect(day >= start && day < end).toBe(true);
          }
        }
      }
    });
  });

  describe("shiftWeekDays", () => {
    it("moves every day one week forward", () => {
      const shifted = shiftWeekDays(getWeekDays(0), 1);
      expect(shifted.map(toLocalISODate)).toEqual(
        getWeekDays(1).map(toLocalISODate),
      );
    });

    it("moves every day one week back", () => {
      const shifted = shiftWeekDays(getWeekDays(0), -1);
      expect(shifted.map(toLocalISODate)).toEqual(
        getWeekDays(-1).map(toLocalISODate),
      );
    });

    it("keeps the days at local midnight across a DST switch", () => {
      const beforeDst = [new Date(2026, 2, 23), new Date(2026, 2, 27)];
      const shifted = shiftWeekDays(beforeDst, 1);
      expect(shifted.map(toLocalISODate)).toEqual(["2026-03-30", "2026-04-03"]);
      expect(shifted.every((day) => day.getHours() === 0)).toBe(true);
    });
  });

  describe("getIsoWeek", () => {
    it("numbers a week by the ISO 8601 rule that week 1 holds the first Thursday", () => {
      expect(getIsoWeek(new Date(2026, 0, 1))).toBe(1);
      expect(getIsoWeek(new Date(2025, 11, 29))).toBe(1);
      expect(getIsoWeek(new Date(2026, 7, 17))).toBe(34);
    });

    it("keeps every day of a week on the same number", () => {
      const numbers = getWeekDays(0, true).map(getIsoWeek);
      expect(numbers).toEqual(Array(7).fill(numbers[0]));
    });

    it("counts the 53rd week of a long year across the year boundary", () => {
      expect(getIsoWeek(new Date(2026, 11, 31))).toBe(53);
      expect(getIsoWeek(new Date(2027, 0, 3))).toBe(53);
      expect(getIsoWeek(new Date(2027, 0, 4))).toBe(1);
    });

    it("assigns the last days of a year to the next year's week 1", () => {
      expect(getIsoWeek(new Date(2024, 11, 30))).toBe(1);
      expect(getIsoWeek(new Date(2022, 11, 31))).toBe(52);
    });

    it("stays stable across a DST switch", () => {
      expect(getIsoWeek(new Date(2026, 2, 30))).toBe(14);
      expect(getIsoWeek(new Date(2026, 9, 26))).toBe(44);
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
