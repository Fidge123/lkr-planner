import {
  afterAll,
  beforeAll,
  describe,
  expect,
  it,
  setSystemTime,
} from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { WeekPicker } from "./week-picker";

function renderPicker(monday: Date): string {
  const weekDays = [0, 1, 2, 3, 4].map(
    (i) =>
      new Date(monday.getFullYear(), monday.getMonth(), monday.getDate() + i),
  );
  return renderToStaticMarkup(
    <WeekPicker weekDays={weekDays} onJumpToWeek={() => {}}>
      KW
    </WeekPicker>,
  );
}

function monthTitles(html: string): string[] {
  return [...html.matchAll(/<h3[^>]*>([^<]+)<\/h3>/g)].map((match) => match[1]);
}

function monthSection(html: string, title: string): string {
  const pattern = new RegExp(
    `<section[^>]*><h3[^>]*>${title}</h3>.*?</section>`,
    "s",
  );
  return html.match(pattern)?.[0] ?? "";
}

function buttonLabelled(html: string, label: string): string {
  return (
    html.match(new RegExp(`<button[^>]*aria-label="${label}"[^>]*>`))?.[0] ?? ""
  );
}

function highlightedWeeks(html: string): string[] {
  return (html.match(/<li[^>]*>.*?<\/li>/gs) ?? [])
    .filter((row) => /^<li[^>]*bg-primary\/10/.test(row))
    .map((row) => row.match(/aria-label="(KW \d+)"/)?.[1] ?? "");
}

describe("WeekPicker", () => {
  beforeAll(() => {
    setSystemTime(new Date(2026, 9, 17, 12));
  });
  afterAll(() => {
    setSystemTime();
  });

  it("renders the picker as a light-dismiss popover opened by its trigger", () => {
    const html = renderPicker(new Date(2026, 9, 12));

    const trigger = buttonLabelled(html, "KW 42, Woche auswählen");
    const targetId = trigger.match(/popovertarget="([^"]+)"/i)?.[1];
    expect(targetId).toBeDefined();
    const picker = html.match(/<section[^>]*popover="auto"[^>]*>/)?.[0] ?? "";
    expect(picker).toContain(`id="${targetId}"`);
  });

  it("shows the displayed week's month on the right and the month before it on the left", () => {
    expect(monthTitles(renderPicker(new Date(2026, 9, 12)))).toEqual([
      "September 2026",
      "Oktober 2026",
    ]);
  });

  it("opens on the month of the Thursday for a week starting in the month before", () => {
    expect(monthTitles(renderPicker(new Date(2026, 8, 28)))).toEqual([
      "September 2026",
      "Oktober 2026",
    ]);
  });

  it("opens across the year boundary with controls to move a month back and forward", () => {
    const html = renderPicker(new Date(2027, 0, 11));

    expect(monthTitles(html)).toEqual(["Dezember 2026", "Januar 2027"]);
    expect(buttonLabelled(html, "Vorheriger Monat")).not.toBe("");
    expect(buttonLabelled(html, "Nächster Monat")).not.toBe("");
  });

  it("lays a month out in Monday-first week rows led by their calendar week", () => {
    const october = monthSection(
      renderPicker(new Date(2026, 9, 12)),
      "Oktober 2026",
    );

    for (const weekday of ["Mo", "Di", "Mi", "Do", "Fr", "Sa", "So"]) {
      expect(october).toContain(`>${weekday}</li>`);
    }
    const labels = [...october.matchAll(/aria-label="([^"]+)"/g)].map(
      (match) => match[1],
    );
    expect(labels.slice(0, 2)).toEqual(["KW 40", "Montag, 28. September 2026"]);
    expect(labels).toContain("KW 44");
    expect(labels[labels.length - 1]).toBe("Sonntag, 1. November 2026");
  });

  it("subdues the days of neighbouring months", () => {
    const october = monthSection(
      renderPicker(new Date(2026, 9, 12)),
      "Oktober 2026",
    );

    expect(buttonLabelled(october, "Montag, 28. September 2026")).toContain(
      "text-base-content/40",
    );
    expect(
      buttonLabelled(october, "Donnerstag, 1. Oktober 2026"),
    ).not.toContain("text-base-content/40");
  });

  it("highlights the displayed week's row", () => {
    expect(highlightedWeeks(renderPicker(new Date(2026, 8, 7)))).toEqual([
      "KW 37",
    ]);
  });

  it("highlights the displayed week in both months when it spans them", () => {
    expect(highlightedWeeks(renderPicker(new Date(2026, 8, 28)))).toEqual([
      "KW 40",
      "KW 40",
    ]);
  });

  it("marks today", () => {
    const html = renderPicker(new Date(2026, 9, 12));

    expect(buttonLabelled(html, "Samstag, 17. Oktober 2026")).toContain(
      'aria-current="date"',
    );
    expect(html.match(/aria-current="date"/g)).toHaveLength(1);
  });
});
