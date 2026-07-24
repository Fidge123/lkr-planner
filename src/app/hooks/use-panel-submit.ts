import { useState } from "react";
import type { PanelStatus } from "../components/settings/panel-status";

export interface PanelSubmitState {
  isSaving: boolean;
  status: PanelStatus | null;
  setStatus: (status: PanelStatus | null) => void;
  run: (
    action: () => Promise<PanelStatus | null>,
    fallbackMessage: string,
  ) => Promise<void>;
}

/// Owns the saving/status lifecycle shared by the settings panels: `run` clears the
/// previous status, marks the panel busy, and turns a thrown error into a German status
/// message, falling back to `fallbackMessage` for non-Error throws. The action returns the
/// success status to show, or null to leave the panel without one.
export function usePanelSubmit(): PanelSubmitState {
  const [isSaving, setIsSaving] = useState(false);
  const [status, setStatus] = useState<PanelStatus | null>(null);

  const run = async (
    action: () => Promise<PanelStatus | null>,
    fallbackMessage: string,
  ) => {
    setIsSaving(true);
    setStatus(null);
    try {
      setStatus(await action());
    } catch (error) {
      setStatus({
        type: "error",
        message: error instanceof Error ? error.message : fallbackMessage,
      });
    } finally {
      setIsSaving(false);
    }
  };

  return { isSaving, status, setStatus, run };
}
