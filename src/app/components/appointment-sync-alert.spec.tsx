import { describe, expect, it } from "bun:test";
import { renderComponent } from "../../test/render-component";
import { AppointmentSyncAlert } from "./appointment-sync-alert";

function renderAlert(failedDates: string[], isRetrying = false): string {
  return renderComponent(
    <AppointmentSyncAlert
      failedDates={failedDates}
      isRetrying={isRetrying}
      onRetry={() => {}}
    />,
  ).container.innerHTML;
}

describe("AppointmentSyncAlert", () => {
  it("renders nothing without failed days", () => {
    expect(renderAlert([])).toBe("");
  });

  it("names the failed days in German date format", () => {
    const html = renderAlert(["2026-11-02", "2026-11-03"]);

    expect(html).toContain("alert alert-error");
    expect(html).toContain(
      "Daylite-Termine für folgende Tage konnten nicht aktualisiert werden: 02.11.2026, 03.11.2026",
    );
  });

  it("offers a retry", () => {
    const html = renderAlert(["2026-11-02"]);

    expect(html).toContain("Erneut versuchen");
    expect(html).not.toContain("disabled");
  });

  it("disables the retry while it runs", () => {
    const html = renderAlert(["2026-11-02"], true);

    expect(html).toContain("Wird wiederholt...");
    expect(html).not.toContain("Erneut versuchen");
    expect(html).toContain('disabled=""');
  });
});
