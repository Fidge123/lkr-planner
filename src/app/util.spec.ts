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
  millisecondsUntilNextLocalMidnight,
  monthWeeks,
  pickerAnchorMonth,
  shiftMonth,
  shiftWeekDays,
  toLocalISODate,
  weekOffsetFor,
} from "./util";

describe("util", () => {
  beforeAll(() => {
    setSystemTime(new Date(2026, 0, 1, 12, 34, 56));
  });

  describe("getWeekDays", () => {
    // Compare local dates: toISOString shifts local midnight a day east of UTC.
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
      expect(weekDays[0].getDay()).toBe(1);
      expect(weekDays[4].getDay()).toBe(5);
    });

    it("returns 7 days from Monday to Sunday when showWeekend is true", () => {
      const weekDays = getWeekDays(0, true);
      expect(weekDays).toHaveLength(7);
      expect(weekDays[0].getDay()).toBe(1);
      expect(weekDays[5].getDay()).toBe(6);
      expect(weekDays[6].getDay()).toBe(0);
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
        expect(weekDays[0].getDay()).toBe(1);
        expect(weekDays.map(toLocalISODate)).toContain("2026-01-03");
        expect(toLocalISODate(weekDays[5])).toBe("2026-01-03");
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
        expect(weekDays[0].getDay()).toBe(1);
        expect(weekDays.map(toLocalISODate)).toContain("2026-01-04");
        expect(toLocalISODate(weekDays[6])).toBe("2026-01-04");
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

  describe("monthWeeks", () => {
    const mondays = (year: number, month: number) =>
      monthWeeks(year, month).map((week) => toLocalISODate(week.days[0]));

    it("starts on the Monday on or before the 1st and ends on the Sunday on or after the last day", () => {
      const weeks = monthWeeks(2026, 9);

      expect(mondays(2026, 9)).toEqual([
        "2026-09-28",
        "2026-10-05",
        "2026-10-12",
        "2026-10-19",
        "2026-10-26",
      ]);
      expect(toLocalISODate(weeks[4].days[6])).toBe("2026-11-01");
      expect(weeks.map((week) => week.weekNumber)).toEqual([
        40, 41, 42, 43, 44,
      ]);
    });

    it("lays every row out from Monday to Sunday at local midnight", () => {
      for (const week of monthWeeks(2026, 9)) {
        expect(week.days.map((day) => day.getDay())).toEqual([
          1, 2, 3, 4, 5, 6, 0,
        ]);
        expect(week.days.every((day) => day.getHours() === 0)).toBe(true);
      }
    });

    it("adds no leading row for a month starting on a Monday", () => {
      expect(mondays(2027, 1)).toEqual([
        "2027-02-01",
        "2027-02-08",
        "2027-02-15",
        "2027-02-22",
      ]);
    });

    it("numbers December's last row by the year that owns its Thursday", () => {
      const lastWeekNumber = (year: number) =>
        monthWeeks(year, 11)
          .map((week) => week.weekNumber)
          .pop();

      expect(lastWeekNumber(2026)).toBe(53);
      expect(lastWeekNumber(2025)).toBe(1);
    });
  });

  describe("pickerAnchorMonth", () => {
    const week = (year: number, month: number, monday: number) =>
      [0, 1, 2, 3, 4].map((i) => new Date(year, month, monday + i));

    it("returns the month of a week lying inside one month", () => {
      expect(pickerAnchorMonth(week(2026, 9, 12))).toEqual({
        year: 2026,
        month: 9,
      });
    });

    it("returns the month of the Thursday for a week starting in the month before", () => {
      expect(pickerAnchorMonth(week(2026, 8, 28))).toEqual({
        year: 2026,
        month: 9,
      });
    });

    it("returns the year of the Thursday for a week spanning the year boundary", () => {
      expect(pickerAnchorMonth(week(2025, 11, 29))).toEqual({
        year: 2026,
        month: 0,
      });
    });
  });

  describe("shiftMonth", () => {
    it("moves a month within the year", () => {
      expect(shiftMonth({ year: 2026, month: 9 }, 1)).toEqual({
        year: 2026,
        month: 10,
      });
      expect(shiftMonth({ year: 2026, month: 9 }, -1)).toEqual({
        year: 2026,
        month: 8,
      });
    });

    it("crosses the year boundary in both directions", () => {
      expect(shiftMonth({ year: 2026, month: 11 }, 1)).toEqual({
        year: 2027,
        month: 0,
      });
      expect(shiftMonth({ year: 2027, month: 0 }, -1)).toEqual({
        year: 2026,
        month: 11,
      });
    });
  });

  describe("weekOffsetFor", () => {
    afterAll(() => {
      setSystemTime(new Date(2026, 0, 1, 12, 34, 56));
    });

    const firstShownDay = (day: Date, showWeekend: boolean) =>
      toLocalISODate(
        getWeekDays(weekOffsetFor(day, showWeekend), showWeekend)[0],
      );

    it("counts whole weeks from the current week on a weekday", () => {
      setSystemTime(new Date(2026, 9, 14, 12));

      expect(weekOffsetFor(new Date(2026, 9, 16), false)).toBe(0);
      expect(weekOffsetFor(new Date(2026, 8, 9), false)).toBe(-5);
      expect(weekOffsetFor(new Date(2026, 9, 18), true)).toBe(0);
      expect(firstShownDay(new Date(2026, 8, 9), false)).toBe("2026-09-07");
    });

    it("shows the chosen week on a Saturday whether the weekend is hidden or shown", () => {
      setSystemTime(new Date(2026, 9, 17, 12));

      expect(weekOffsetFor(new Date(2026, 9, 17), false)).toBe(-1);
      expect(weekOffsetFor(new Date(2026, 9, 17), true)).toBe(0);
      expect(firstShownDay(new Date(2026, 9, 17), false)).toBe("2026-10-12");
      expect(firstShownDay(new Date(2026, 9, 21), false)).toBe("2026-10-19");
    });

    it("shows the chosen week on a Sunday whether the weekend is hidden or shown", () => {
      setSystemTime(new Date(2026, 9, 18, 12));

      expect(weekOffsetFor(new Date(2026, 8, 13), false)).toBe(-6);
      expect(weekOffsetFor(new Date(2026, 8, 13), true)).toBe(-5);
      expect(firstShownDay(new Date(2026, 8, 13), false)).toBe("2026-09-07");
      expect(firstShownDay(new Date(2026, 8, 13), true)).toBe("2026-09-07");
    });

    it("counts whole weeks across a DST switch", () => {
      setSystemTime(new Date(2026, 9, 14, 12));

      expect(weekOffsetFor(new Date(2026, 10, 4), false)).toBe(3);
      expect(weekOffsetFor(new Date(2026, 2, 25), false)).toBe(-29);
      expect(firstShownDay(new Date(2026, 2, 25), false)).toBe("2026-03-23");
    });

    it("counts whole weeks across the year boundary", () => {
      setSystemTime(new Date(2026, 11, 30, 12));

      expect(weekOffsetFor(new Date(2027, 0, 6), false)).toBe(1);
      expect(weekOffsetFor(new Date(2027, 0, 1), true)).toBe(0);
      expect(weekOffsetFor(new Date(2026, 11, 20), false)).toBe(-2);
      expect(firstShownDay(new Date(2027, 0, 6), false)).toBe("2027-01-04");
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

  describe("millisecondsUntilNextLocalMidnight", () => {
    it("counts the time left in the current local day", () => {
      expect(
        millisecondsUntilNextLocalMidnight(new Date(2026, 0, 1, 23, 59, 59)),
      ).toBe(1000);
    });

    it("lands on the next local midnight, DST switches included", () => {
      for (const start of [
        new Date(2026, 0, 1, 12, 34, 56),
        new Date(2026, 2, 29, 0, 0, 0),
        new Date(2026, 9, 25, 0, 0, 0),
      ]) {
        const next = new Date(
          start.getTime() + millisecondsUntilNextLocalMidnight(start),
        );
        expect(toLocalISODate(next)).toBe(
          toLocalISODate(
            new Date(
              start.getFullYear(),
              start.getMonth(),
              start.getDate() + 1,
            ),
          ),
        );
        expect(next.getHours()).toBe(0);
        expect(next.getMinutes()).toBe(0);
      }
    });
  });
});
