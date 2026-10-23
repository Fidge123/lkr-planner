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

/// `fallbackMessage` applies only to non-Error throws, because panels show `error.message` directly.
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
