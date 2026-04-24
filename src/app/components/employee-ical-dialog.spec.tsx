import { describe, expect, it } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import type { PlanningContactRecord, ZepCalendar } from "../../generated/tauri";
import { CalendarSection, EmployeeIcalDialog } from "./employee-ical-dialog";

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

// ── 10.8 / 10.9 – CalendarSection unit tests ──────────────────────────────────
// CalendarSection is tested in isolation so that submitting/idle states can be
// controlled directly via props, proving the two sections are fully independent.

describe("CalendarSection (10.8 – independent state)", () => {
  it("primary submitting does not affect the absence section", () => {
    const primaryHtml = renderToStaticMarkup(
      <CalendarSection
        title="Einsatz"
        source="primary"
        calendars={CALENDARS}
        selectedUrl={CALENDARS[0].url}
        storedUrl={CALENDARS[0].url}
        onUrlChange={() => {}}
        status={null}
        isSubmitting={true}
        onSubmit={() => {}}
        isDisabled={false}
      />,
    );
    const absenceHtml = renderToStaticMarkup(
      <CalendarSection
        title="Abwesenheit"
        source="absence"
        calendars={CALENDARS}
        selectedUrl={CALENDARS[0].url}
        storedUrl={CALENDARS[0].url}
        onUrlChange={() => {}}
        status={null}
        isSubmitting={false}
        onSubmit={() => {}}
        isDisabled={false}
        isOptional
      />,
    );

    // Primary is in-flight
    expect(primaryHtml).toContain("Teste...");
    // Absence remains at rest — its own isSubmitting is false
    expect(absenceHtml).toContain("Speichern");
    expect(absenceHtml).not.toContain("Teste...");
  });
});

describe("CalendarSection – clear calendar", () => {
  it("shows 'Entfernen' button when no URL is selected but one was previously stored", () => {
    const html = renderToStaticMarkup(
      <CalendarSection
        title="Einsatz"
        source="primary"
        calendars={CALENDARS}
        selectedUrl=""
        storedUrl={CALENDARS[0].url}
        onUrlChange={() => {}}
        status={null}
        isSubmitting={false}
        onSubmit={() => {}}
        isDisabled={false}
      />,
    );

    expect(html).toContain("Entfernen");
    expect(html).not.toContain("Speichern");
    expect(html).not.toMatch(/disabled/);
  });

  it("disables the button when no URL is selected and no URL was stored", () => {
    const html = renderToStaticMarkup(
      <CalendarSection
        title="Einsatz"
        source="primary"
        calendars={CALENDARS}
        selectedUrl=""
        storedUrl=""
        onUrlChange={() => {}}
        status={null}
        isSubmitting={false}
        onSubmit={() => {}}
        isDisabled={false}
      />,
    );

    expect(html).toMatch(/disabled/);
  });
});

describe("CalendarSection (10.9 – in-flight state)", () => {
  it("shows 'Teste...' and disables the button while isSubmitting=true", () => {
    const html = renderToStaticMarkup(
      <CalendarSection
        title="Einsatz"
        source="primary"
        calendars={CALENDARS}
        selectedUrl={CALENDARS[0].url}
        storedUrl={CALENDARS[0].url}
        onUrlChange={() => {}}
        status={null}
        isSubmitting={true}
        onSubmit={() => {}}
        isDisabled={false}
      />,
    );

    expect(html).toContain("Teste...");
    expect(html).not.toContain("Speichern");
    // The submit button must carry the disabled attribute
    expect(html).toMatch(/disabled/);
  });

  it("shows 'Speichern & Testen' and enabled button when idle", () => {
    const html = renderToStaticMarkup(
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
      />,
    );

    expect(html).toContain("Speichern");
    expect(html).not.toContain("Teste...");
    // No disabled attribute on the submit button (selectedUrl is set, isDisabled=false)
    expect(html).not.toMatch(/disabled/);
  });
});

// ── 10.10 – Discovery failure ─────────────────────────────────────────────────

describe("EmployeeIcalDialog (10.10 – discovery failure)", () => {
  it("shows error banner with reload button when calendar discovery failed", () => {
    const errorMessage = "Verbindung zum ZEP-Server fehlgeschlagen.";
    const html = renderToStaticMarkup(
      <EmployeeIcalDialog
        employee={EMPLOYEE}
        employeeSetting={null}
        onClose={() => {}}
        onSettingsSaved={() => {}}
        zepCalendars={null}
        isLoadingCalendars={false}
        calendarsError={errorMessage}
        onReloadCalendars={() => {}}
      />,
    );

    // Error message is displayed
    expect(html).toContain(errorMessage);
    // A reload button is present
    expect(html).toContain("Neu laden");
    // The dialog itself still renders
    expect(html).toContain("<dialog");
    expect(html).toContain("iCal-Konfiguration");
  });

  it("disables calendar sections while discovery error is shown (zepCalendars=null)", () => {
    const html = renderToStaticMarkup(
      <EmployeeIcalDialog
        employee={EMPLOYEE}
        employeeSetting={null}
        onClose={() => {}}
        onSettingsSaved={() => {}}
        zepCalendars={null}
        isLoadingCalendars={false}
        calendarsError="Fehler"
        onReloadCalendars={() => {}}
      />,
    );

    // Both selects must be disabled (isDisabled = zepCalendars === null)
    const selectMatches = [...html.matchAll(/<select[^>]*disabled[^>]*>/g)];
    expect(selectMatches.length).toBe(2);
  });

  it("renders nothing when no employee is selected", () => {
    const html = renderToStaticMarkup(
      <EmployeeIcalDialog
        employee={null}
        employeeSetting={null}
        onClose={() => {}}
        onSettingsSaved={() => {}}
        zepCalendars={null}
        isLoadingCalendars={false}
        calendarsError={null}
        onReloadCalendars={() => {}}
      />,
    );

    expect(html).toBe("");
  });
});
