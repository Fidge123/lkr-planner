import { type ChangeEvent, useEffect, useState } from "react";
import type { DisplaySettings } from "../../../generated/tauri";
import {
  loadDisplaySettings,
  saveDisplaySettings,
} from "../../../services/display-settings";
import { type PanelStatus, StatusAlert } from "./panel-status";

export function DisplaySettingsPanel({ onClose, onChanged }: Props) {
  const [hideNonPlannable, setHideNonPlannable] = useState(true);
  const [showWeekend, setShowWeekend] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [status, setStatus] = useState<PanelStatus | null>(null);

  useEffect(() => {
    let isActive = true;
    void loadDisplaySettings()
      .then((settings) => {
        if (isActive) {
          setHideNonPlannable(settings.hideNonPlannableEmployees);
          setShowWeekend(settings.showWeekend);
        }
      })
      .catch(() => {
        // The initial state already matches the backend defaults.
      });
    return () => {
      isActive = false;
    };
  }, []);

  const saveToggle =
    (key: keyof DisplaySettings, applyValue: (value: boolean) => void) =>
    async (event: ChangeEvent<HTMLInputElement>) => {
      const nextValue = event.target.checked;
      applyValue(nextValue);
      setIsSaving(true);
      setStatus(null);

      try {
        await saveDisplaySettings({ [key]: nextValue });
        onChanged?.();
      } catch (error) {
        // Revert the optimistic change so the UI matches the persisted state.
        applyValue(!nextValue);
        setStatus({
          type: "error",
          message:
            error instanceof Error
              ? error.message
              : "Die Anzeige-Einstellung konnte nicht gespeichert werden.",
        });
      } finally {
        setIsSaving(false);
      }
    };

  return (
    <>
      <h3 className="text-lg font-semibold">Anzeige</h3>

      <StatusAlert status={status} />

      <label className="label mt-4 cursor-pointer items-start justify-start gap-3">
        <input
          type="checkbox"
          className="toggle toggle-primary"
          checked={hideNonPlannable}
          onChange={saveToggle(
            "hideNonPlannableEmployees",
            setHideNonPlannable,
          )}
          disabled={isSaving}
        />
        <span className="label-text font-medium">
          Nicht planbare Mitarbeiter ausblenden
        </span>
      </label>

      <label className="label mt-4 cursor-pointer items-start justify-start gap-3">
        <input
          type="checkbox"
          className="toggle toggle-primary"
          checked={showWeekend}
          onChange={saveToggle("showWeekend", setShowWeekend)}
          disabled={isSaving}
        />
        <span className="label-text font-medium">Wochenende anzeigen</span>
      </label>

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
  onChanged?: () => void;
}
