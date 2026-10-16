import { type ChangeEvent, useEffect, useState } from "react";
import {
  loadTelemetrySettings,
  saveTelemetryEnabled,
} from "../../../services/telemetry";
import { usePanelSubmit } from "../../hooks/use-panel-submit";
import { StatusAlert } from "./panel-status";

export function TelemetrySettingsPanel({ onClose }: Props) {
  const [isEnabled, setIsEnabled] = useState(false);
  const { isSaving, status, run } = usePanelSubmit();

  useEffect(() => {
    let isActive = true;
    void loadTelemetrySettings()
      .then((enabled) => {
        if (isActive) {
          setIsEnabled(enabled);
        }
      })
      .catch(() => {});
    return () => {
      isActive = false;
    };
  }, []);

  const saveToggle = async (event: ChangeEvent<HTMLInputElement>) => {
    const nextValue = event.target.checked;
    setIsEnabled(nextValue);

    await run(async () => {
      try {
        await saveTelemetryEnabled(nextValue);
      } catch (error) {
        setIsEnabled(!nextValue);
        throw error;
      }
      return null;
    }, "Die Diagnose-Einstellung konnte nicht gespeichert werden.");
  };

  return (
    <>
      <h3 className="text-lg font-semibold">Diagnose</h3>

      <StatusAlert status={status} />

      <label className="label mt-4 cursor-pointer items-start justify-start gap-3">
        <input
          type="checkbox"
          className="toggle toggle-primary"
          checked={isEnabled}
          onChange={saveToggle}
          disabled={isSaving}
        />
        <span className="label-text font-medium">Diagnosedaten senden</span>
      </label>

      <p className="mt-4 text-sm text-base-content/70">
        Es werden Fehler- und Leistungsdaten an den Analysedienst PostHog
        (Server in der EU) übertragen, damit Störungen und langsame Abläufe
        erkannt und behoben werden können.
      </p>

      <p className="mt-2 text-sm text-base-content/70">
        Fehlerberichte enthalten technische Details wie Fehlermeldungen und
        aufgerufene Adressen und können Projekt- oder Mitarbeiternamen
        enthalten. Zusätzlich wird die IP-Adresse übermittelt.
      </p>

      <p className="mt-2 text-sm text-base-content/70">
        Passwörter, Zugangsschlüssel und Tokens werden nie übertragen.
      </p>

      <section className="mt-6 flex items-center justify-end">
        <button type="button" className="btn btn-sm" onClick={onClose}>
          Schließen
        </button>
      </section>
    </>
  );
}

interface Props {
  onClose: () => void;
}
