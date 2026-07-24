import { type ChangeEvent, useEffect, useState } from "react";
import type { DisplaySettings } from "../../../generated/tauri";
import {
  loadDisplaySettings,
  saveDisplaySettings,
} from "../../../services/display-settings";
import { usePanelSubmit } from "../../hooks/use-panel-submit";
import { StatusAlert } from "./panel-status";

export function DisplaySettingsPanel({ onClose, onChanged }: Props) {
  const [hideNonPlannable, setHideNonPlannable] = useState(true);
  const [showWeekend, setShowWeekend] = useState(false);
  const { isSaving, status, run } = usePanelSubmit();

  useEffect(() => {
    let isActive = true;
    void loadDisplaySettings()
      .then((settings) => {
        if (isActive) {
          setHideNonPlannable(settings.hideNonPlannableEmployees);
          setShowWeekend(settings.showWeekend);
        }
      })
      .catch(() => {});
    return () => {
      isActive = false;
    };
  }, []);

  const saveToggle =
    (key: keyof DisplaySettings, applyValue: (value: boolean) => void) =>
    async (event: ChangeEvent<HTMLInputElement>) => {
      const nextValue = event.target.checked;
      applyValue(nextValue);

      await run(async () => {
        try {
          await saveDisplaySettings({ [key]: nextValue });
        } catch (error) {
          // Revert the optimistic toggle before the shared handler maps the error.
          applyValue(!nextValue);
          throw error;
        }
        onChanged?.();
        return null;
      }, "Die Anzeige-Einstellung konnte nicht gespeichert werden.");
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
