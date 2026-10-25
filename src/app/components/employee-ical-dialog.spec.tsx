import { describe, expect, it } from "bun:test";
import type { PlanningContactRecord, ZepCalendar } from "../../generated/tauri";
import { renderComponent } from "../../test/render-component";
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
    ...overrides,
  };
}

const zepCalendars: ZepCalendar[] = [
  { displayName: "Team-Kalender", url: "https://zep.example.com/calendars/1" },
  { displayName: "Urlaub", url: "https://zep.example.com/calendars/2" },
];

const ownEmployee: PlanningContactRecord = {
  self: "/contacts/42",
  full_name: "Max Mustermann",
  nickname: null,
  category: null,
  urls: [],
};

function renderSection(
  overrides: Partial<Parameters<typeof CalendarSection>[0]> = {},
): string {
  return renderComponent(
    <CalendarSection
      title="Einsatz"
      source="primary"
      calendars={zepCalendars}
      selectedUrl={zepCalendars[0].url}
      storedUrl={zepCalendars[0].url}
      onUrlChange={() => {}}
      status={null}
      isSubmitting={false}
      onSubmit={() => {}}
      isDisabled={false}
      {...overrides}
    />,
  ).container.innerHTML;
}

function renderDialog(
  overrides: Partial<Parameters<typeof EmployeeIcalDialog>[0]> = {},
): string {
  return renderComponent(
    <EmployeeIcalDialog
      employee={ownEmployee}
      employeeSetting={null}
      onClose={() => {}}
      onSettingsSaved={() => {}}
      calendarState={calendarState()}
      {...overrides}
    />,
  ).container.innerHTML;
}

describe("CalendarSection", () => {
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
