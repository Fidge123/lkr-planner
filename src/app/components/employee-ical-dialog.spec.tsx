import { describe, expect, it } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import type { PlanningContactRecord, ZepCalendar } from "../../generated/tauri";
import type { ZepCalendarsState } from "../hooks/use-zep-calendars";
import { CalendarSection } from "./calendar-section";
import { EmployeeIcalDialog } from "./employee-ical-dialog";

function calendarState(
  overrides: Partial<ZepCalendarsState> = {},
): ZepCalendarsState {
  return {
    calendars: null,
    isLoading: false,
    errorMessage: null,
    reload: () => {},
    ensureLoaded: () => {},
    ...overrides,
  };
}

const CALENDARS: ZepCalendar[] = [
  { displayName: "Team-Kalender", url: "https://zep.example.com/calendars/1" },
  { displayName: "Urlaub", url: "https://zep.example.com/calendars/2" },
];

const EMPLOYEE: PlanningContactRecord = {
  self: "/contacts/42",
  full_name: "Max Mustermann",
  nickname: null,
  category: null,
  urls: [],
};

function renderSection(
  overrides: Partial<Parameters<typeof CalendarSection>[0]> = {},
): string {
  return renderToStaticMarkup(
    <CalendarSection
      title="Einsatz"
      source="primary"
      calendars={CALENDARS}
      selectedUrl={CALENDARS[0].url}
      storedUrl={CALENDARS[0].url}
      onUrlChange={() => {}}
      status={null}
      isSubmitting={false}
      onSubmit={() => {}}
      isDisabled={false}
      {...overrides}
    />,
  );
}

function renderDialog(
  overrides: Partial<Parameters<typeof EmployeeIcalDialog>[0]> = {},
): string {
  return renderToStaticMarkup(
    <EmployeeIcalDialog
      employee={EMPLOYEE}
      employeeSetting={null}
      onClose={() => {}}
      onSettingsSaved={() => {}}
      calendarState={calendarState()}
      {...overrides}
    />,
  );
}

describe("CalendarSection", () => {
  it("submitting one section does not affect the other", () => {
    const primaryHtml = renderSection({ isSubmitting: true });
    const absenceHtml = renderSection({
      title: "Abwesenheit",
      source: "absence",
      isOptional: true,
    });

    expect(primaryHtml).toContain("Teste...");
    expect(absenceHtml).toContain("Speichern");
    expect(absenceHtml).not.toContain("Teste...");
  });

  it("shows 'Teste...' and disables the button while submitting", () => {
    const html = renderSection({ isSubmitting: true });

    expect(html).toContain("Teste...");
    expect(html).not.toContain("Speichern");
    expect(html).toMatch(/disabled/);
  });

  it("shows 'Speichern & Testen' and an enabled button when idle", () => {
    const html = renderSection();

    expect(html).toContain("Speichern");
    expect(html).not.toContain("Teste...");
    expect(html).not.toMatch(/disabled/);
  });

  it("offers 'Entfernen' when the stored URL is being cleared", () => {
    const html = renderSection({ selectedUrl: "" });

    expect(html).toContain("Entfernen");
    expect(html).not.toContain("Speichern");
    expect(html).not.toMatch(/disabled/);
  });

  it("disables the button when no URL is selected and none was stored", () => {
    expect(renderSection({ selectedUrl: "", storedUrl: "" })).toMatch(
      /disabled/,
    );
  });
});

describe("EmployeeIcalDialog", () => {
  it("shows an error banner with a reload button when discovery failed", () => {
    const errorMessage = "Verbindung zum ZEP-Server fehlgeschlagen.";
    const html = renderDialog({
      calendarState: calendarState({ errorMessage }),
    });

    expect(html).toContain(errorMessage);
    expect(html).toContain("Neu laden");
    expect(html).toContain("<dialog");
    expect(html).toContain("iCal-Konfiguration");
  });

  it("disables both calendar sections while the discovery error is shown", () => {
    const html = renderDialog({
      calendarState: calendarState({ errorMessage: "Fehler" }),
    });

    expect([...html.matchAll(/<select[^>]*disabled[^>]*>/g)]).toHaveLength(2);
  });

  it("renders nothing when no employee is selected", () => {
    expect(renderDialog({ employee: null })).toBe("");
  });
});
