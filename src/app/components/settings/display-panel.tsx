import { useQueryClient } from "@tanstack/react-query";
import type { ChangeEvent } from "react";
import type { DisplaySettings } from "../../../generated/tauri";
import { saveDisplaySettings } from "../../../services/display-settings";
import { localStoreQuery, useLocalStore } from "../../hooks/use-local-store";
import { usePanelSubmit } from "../../hooks/use-panel-submit";
import { StatusAlert } from "./panel-status";

export function DisplaySettingsPanel({ onClose }: Props) {
  const queryClient = useQueryClient();
  const { isLoaded, displaySettings } = useLocalStore();
  const { isSaving, status, run } = usePanelSubmit();

  const saveToggle =
    (key: keyof DisplaySettings) => (event: ChangeEvent<HTMLInputElement>) => {
      const next = { ...displaySettings, [key]: event.target.checked };
      queryClient.setQueryData(
        localStoreQuery.queryKey,
        (store) => store && { ...store, displaySettings: next },
      );
      void run(async () => {
        try {
          await saveDisplaySettings(next);
        } finally {
          await queryClient.invalidateQueries({
            queryKey: localStoreQuery.queryKey,
          });
        }
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
          checked={displaySettings.hideNonPlannableEmployees}
          onChange={saveToggle("hideNonPlannableEmployees")}
          disabled={isSaving || !isLoaded}
        />
        <span className="label-text font-medium">
          Nicht planbare Mitarbeiter ausblenden
        </span>
      </label>

      <label className="label mt-4 cursor-pointer items-start justify-start gap-3">
        <input
          type="checkbox"
          className="toggle toggle-primary"
          checked={displaySettings.showWeekend}
          onChange={saveToggle("showWeekend")}
          disabled={isSaving || !isLoaded}
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
}
